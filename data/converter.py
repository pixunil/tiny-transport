#!/usr/bin/env python3

from operator import attrgetter
import csv
import json

class Location(object):
    def __init__(self, id, name, lat, lon):
        self.id = id
        self.name = name
        self.lat = lat
        self.lon = lon
        self.stops = []

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

    @property
    def is_suburban_railway(self):
        return self.type == 109

    @staticmethod
    def fromCsv(routeCsv):
        reader = csv.DictReader(routeCsv)

        routes = {}
        for row in reader:
            route = Route(row["route_id"], row["route_short_name"], int(row["route_type"]))
            routes[route.id] = route

        return routes

class Trip(object):
    def __init__(self, id, route):
        self.id = id
        self.route = route
        self.stops = []

    @staticmethod
    def fromCsv(tripCsv, routes):
        reader = csv.DictReader(tripCsv)

        trips = {}
        for row in reader:
            trip = Trip(row["trip_id"], routes[row["route_id"]])
            trips[trip.id] = trip

        return trips

class Stop(object):
    def __init__(self, trip, location, order):
        self.trip = trip
        self.location = location
        self.order = order
        self.location.stops.append(self)
        self.trip.stops.append(self)

    @staticmethod
    def fromCsv(stopCsv, trips, locations):
        reader = csv.DictReader(stopCsv)

        for row in reader:
            Stop(trips[row["trip_id"]], locations[row["stop_id"]], int(row["stop_sequence"]))

        for trip in trips.values():
            trip.stops.sort(key=attrgetter("order"))

if __name__ == "__main__":
    print("Parsing locations...")
    with open("vbb/stops.txt") as locationCsv:
        locations = Location.fromCsv(locationCsv)

    print("Parsing routes...")
    with open("vbb/routes.txt") as routeCsv:
        routes = Route.fromCsv(routeCsv)

    print("Parsing trips...")
    with open("vbb/trips.txt") as tripCsv:
        trips = Trip.fromCsv(tripCsv, routes)

    print("Parsing stops...")
    with open("vbb/stop_times.txt") as stopCsv:
        Stop.fromCsv(stopCsv, trips, locations)

    print("Fetching locations...")
    data = []
    for location in locations.values():
        if any(stop.trip.route.is_suburban_railway for stop in location.stops):
            data.append({
                "lat": location.lat,
                "lon": location.lon,
            })

    print("Exporting...")
    with open("vbb.json", "w") as jsonfile:
        json.dump(data, jsonfile)
