#!/usr/bin/env python3

import os
from operator import attrgetter
from datetime import datetime
import argparse
import csv
import json
import pickle

class DataSet(object):
    def loadData(self):
        print("Parsing agencies...")
        with open("vbb/agency.txt") as agencyCsv:
            (self.agencies, agenciesById) = Agency.fromCsv(agencyCsv)

        print("Parsing locations...")
        with open("vbb/stops.txt") as locationCsv:
            (self.locations, locationsById) = Location.fromCsv(locationCsv)

        print("Parsing services...")
        with open("vbb/calendar.txt") as calendarCsv, open("vbb/calendar_dates.txt") as calendarDatesCsv:
            (self.services, _servicesById) = Service.fromCsv(calendarCsv, calendarDatesCsv)

        print("Parsing routes...")
        with open("vbb/routes.txt") as routeCsv:
            (self.routes, routesById) = Route.fromCsv(routeCsv, agenciesById)

        print("Parsing trips...")
        with open("vbb/trips.txt") as tripCsv:
            (self.trips, tripsById) = Trip.fromCsv(tripCsv, routesById)

        print("Parsing stops...")
        with open("vbb/stop_times.txt") as stopCsv:
            Stop.fromCsv(stopCsv, tripsById, locationsById)

        print("Parsing colors...")
        with open("vbb/colors.csv") as colorsCsv:
            Route.addColorsFromCsv(colorsCsv, routesById)

class Service(object):
    def __init__(self, start, end, available_weekdays):
        self.start = start
        self.end = end
        self.available_weekdays = available_weekdays
        self.added = set()
        self.removed = set()

    def __repr__(self):
        return "".join(name if available else "-" for available, name in zip(self.available_weekdays, "MTWTFSS"))

    def available_at(self, date):
        return date not in self.removed and (date in self.added or self.regulary_available_at(date))

    def regulary_available_at(self, date):
        return self.start <= date <= self.end and self.available_weekdays[date.weekday()]

    @staticmethod
    def fromCsv(calendarCsv, calendarDatesCsv):
        reader = csv.DictReader(calendarCsv)

        servicesById = {}
        for row in reader:
            start = datetime.strptime(row["start_date"], "%Y%m%d").date()
            end = datetime.strptime(row["end_date"], "%Y%m%d").date()
            available_weekdays = (row[day] == "1" for day in ("monday", "tuesday", "wednesday", "thursday", "friday", "saturday", "sunday"))
            service = Service(start, end, tuple(available_weekdays))
            servicesById[row["service_id"]] = service

        reader = csv.DictReader(calendarDatesCsv)
        for row in reader:
            service = servicesById[row["service_id"]]
            date = datetime.strptime(row["date"], "%Y%m%d").date()
            if row["exception_type"] == "1":
                service.added.add(date)
            else:
                service.removed.add(date)

        return (list(servicesById.values()), servicesById)

class Agency(object):
    def __init__(self, name):
        self.name = name
        self.routes = []

    def __repr__(self):
        return self.name

    def __eq__(self, other):
        return self.name == other.name

    def __hash__(self):
        return hash(self.name)

    @staticmethod
    def fromCsv(agencyCsv):
        reader = csv.DictReader(agencyCsv)

        agenciesById = {}
        for row in reader:
            agency = Agency(row["agency_name"])
            agenciesById[row["agency_id"]] = agency

        return (list(agenciesById.values()), agenciesById)

class Location(object):
    def __init__(self, name, lat, lon):
        self.name = name
        self.lat = lat
        self.lon = lon

    def __repr__(self):
        return self.name

    @staticmethod
    def fromCsv(locationCsv):
        reader = csv.DictReader(locationCsv)

        locationsById = {}
        for row in reader:
            location = Location(row["stop_name"], float(row["stop_lat"]), float(row["stop_lon"]))
            locationsById[row["stop_id"]] = location

        return (list(locationsById.values()), locationsById)

class Route(object):
    def __init__(self, agency, name, type):
        self.agency = agency
        self.name = name
        self.type = type
        self.trips = []

    def __eq__(self, other):
        return self.agency == other.agency and self.name == other.name and self.type == other.type

    def __hash__(self):
        return hash((self.agency, self.name, self.type))

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
        routesById = {}
        for row in reader:
            route = Route(agencies[row["agency_id"]], row["route_short_name"], int(row["route_type"]))

            if route in routes:
                route = routes[route]
            else:
                routes[route] = route
                route.agency.routes.append(route)

            routesById[row["route_id"]] = route

        return (list(routes.values()), routesById)

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

        tripsById = {}
        for row in reader:
            trip = Trip(row["trip_id"])
            routes[row["route_id"]].trips.append(trip)
            tripsById[trip.id] = trip

        return (list(tripsById.values()), tripsById)

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

parser = argparse.ArgumentParser()
parser.add_argument("--refresh", action="store_true")

if __name__ == "__main__":
    args = parser.parse_args()

    if not args.refresh and os.path.exists("vbb.pickle"):
        print("Loading dataset...")
        with open("vbb.pickle", "rb") as pickleFile:
            dataset = pickle.load(pickleFile)
    else:
        dataset = DataSet()
        dataset.loadData()
        with open("vbb.pickle", "wb") as pickleFile:
            pickle.dump(dataset, pickleFile)

    print("Fetching data...")
    locations = set()
    routes = []

    for agency in dataset.agencies:
        if agency.name == "S-Bahn Berlin GmbH":
            for route in agency.routes:
                if route.is_suburban_railway:
                    trip = max(route.trips, key=lambda trip: len(trip.stops))
                    locations |= set(stop.location for stop in trip.stops)
                    routes.append({
                        "color": route.color,
                        "stops": trip.stops,
                    })

    locations = list(locations)
    data = {
        "routes": [],
        "locations": [],
    }
    for route in routes:
        data["routes"].append({
            "color": route["color"],
            "stops": [locations.index(stop.location) for stop in route["stops"]]
        })

    for location in locations:
        data["locations"].append({
            "lat": location.lat,
            "lon": location.lon,
        })

    print("Exporting...")
    with open("vbb.json", "w") as jsonfile:
        json.dump(data, jsonfile)
