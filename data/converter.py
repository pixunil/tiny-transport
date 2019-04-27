#!/usr/bin/env python3

import os
from operator import attrgetter
import csv
import json
import pickle

class DataSet(object):
    def loadData(self):
        print("Parsing agencies...")
        with open("vbb/agency.txt") as agencyCsv:
            self.agencies = Agency.fromCsv(agencyCsv)

        print("Parsing locations...")
        with open("vbb/stops.txt") as locationCsv:
            self.locations = Location.fromCsv(locationCsv)

        print("Parsing routes...")
        with open("vbb/routes.txt") as routeCsv:
            self.routes = Route.fromCsv(routeCsv, self.agencies)

        print("Parsing trips...")
        with open("vbb/trips.txt") as tripCsv:
            self.trips = Trip.fromCsv(tripCsv, self.routes)

        print("Parsing stops...")
        with open("vbb/stop_times.txt") as stopCsv:
            Stop.fromCsv(stopCsv, self.trips, self.locations)

        print("Parsing colors...")
        with open("vbb/colors.csv") as colorsCsv:
            Route.addColorsFromCsv(colorsCsv, self.routes)

class Agency(object):
    def __init__(self, id, name):
        self.id = id
        self.name = name
        self.routes = []

    def __repr__(self):
        return self.name

    @staticmethod
    def fromCsv(agencyCsv):
        reader = csv.DictReader(agencyCsv)

        agencies = {}
        for row in reader:
            agency = Agency(row["agency_id"], row["agency_name"])
            agencies[agency.id] = agency

        return agencies

class Location(object):
    def __init__(self, id, name, lat, lon):
        self.id = id
        self.name = name
        self.lat = lat
        self.lon = lon

    def __repr__(self):
        return self.name

    @staticmethod
    def fromCsv(locationCsv):
        reader = csv.DictReader(locationCsv)

        locations = {}
        for row in reader:
            location = Location(row["stop_id"], row["stop_name"], float(row["stop_lat"]), float(row["stop_lon"]))
            locations[location.id] = location

        return locations

class Route(object):
    def __init__(self, id, name, type):
        self.id = id
        self.name = name
        self.type = type
        self.trips = []

    def __repr__(self):
        return self.name

    @property
    def is_railway(self):
        return self.type == 100

    @property
    def is_suburban_railway(self):
        return self.type == 109

    @property
    def is_urban_railway(self):
        return self.type == 400

    @property
    def should_have_color(self):
        return self.is_railway or self.is_suburban_railway or self.is_urban_railway

    @staticmethod
    def fromCsv(routeCsv, agencies):
        reader = csv.DictReader(routeCsv)

        routes = {}
        for row in reader:
            route = Route(row["route_id"], row["route_short_name"], int(row["route_type"]))
            agencies[row["agency_id"]].routes.append(route)
            routes[route.id] = route

        return routes

    @staticmethod
    def addColorsFromCsv(colorCsv, routes):
        reader = csv.DictReader(colorCsv, delimiter=";")

        for row in reader:
            for route in routes.values():
                if route.name == row["Linie"] and route.should_have_color:
                    route.color = [int(row[component]) for component in "RGB"]

class Trip(object):
    def __init__(self, id):
        self.id = id
        self.stops = []

    @staticmethod
    def fromCsv(tripCsv, routes):
        reader = csv.DictReader(tripCsv)

        trips = {}
        for row in reader:
            trip = Trip(row["trip_id"])
            routes[row["route_id"]].trips.append(trip)
            trips[trip.id] = trip

        return trips

class Stop(object):
    def __init__(self, location, order):
        self.location = location
        self.order = order

    def __repr__(self):
        return repr(self.location)

    @staticmethod
    def fromCsv(stopCsv, trips, locations):
        reader = csv.DictReader(stopCsv)

        for row in reader:
            stop = Stop(locations[row["stop_id"]], int(row["stop_sequence"]))
            trips[row["trip_id"]].stops.append(stop)

        for trip in trips.values():
            trip.stops.sort(key=attrgetter("order"))

if __name__ == "__main__":
    if os.path.exists("vbb.pickle"):
        print("Loading dataset...")
        with open("vbb.pickle", "rb") as pickleFile:
            dataset = pickle.load(pickleFile)
    else:
        dataset = DataSet()
        dataset.loadData()
        with open("vbb.pickle", "wb") as pickleFile:
            pickle.dump(dataset, pickleFile)

    print("Fetching data...")
    data = {
        "routes": [],
        "locations": {},
    }

    for agency in dataset.agencies.values():
        if agency.name == "S-Bahn Berlin GmbH":
            for route in agency.routes:
                if route.is_suburban_railway:
                    trip = max(route.trips, key=lambda trip: len(trip.stops))

                    data["routes"].append({
                        "color": route.color,
                        "stops": [stop.location.id for stop in trip.stops]
                    })

                    for stop in trip.stops:
                        location = stop.location
                        data["locations"][location.id] = {
                            "lat": location.lat,
                            "lon": location.lon,
                        }

    print("Exporting...")
    with open("vbb.json", "w") as jsonfile:
        json.dump(data, jsonfile)
