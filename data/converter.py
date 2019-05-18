#!/usr/bin/env python3

import os
from operator import itemgetter
import datetime
import argparse
import csv
import json
import pickle

def parse_timedelta(time):
    parts = [int(part) for part in time.split(":")]
    return datetime.timedelta(hours=parts[0], minutes=parts[1], seconds=parts[2])

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
            (self.services, servicesById) = Service.fromCsv(calendarCsv, calendarDatesCsv)

        print("Parsing lines...")
        with open("vbb/routes.txt") as routeCsv:
            (self.lines, linesById) = Line.fromCsv(routeCsv, agenciesById)

        print("Parsing trips...")
        with open("vbb/trips.txt") as tripCsv, open("vbb/stop_times.txt") as stopCsv:
            Trip.fromCsv(tripCsv, stopCsv, locationsById, linesById, servicesById)

        print("Parsing colors...")
        with open("vbb/colors.csv") as colorsCsv:
            Line.addColorsFromCsv(colorsCsv, self.lines)

class Service(object):
    def __init__(self, start, end, available_weekdays):
        self.start = start
        self.end = end
        self.available_weekdays = available_weekdays
        self.added = set()
        self.removed = set()

    def __repr__(self):
        days = "".join(name if available else "-" for available, name in zip(self.available_weekdays, "MTWTFSS"))
        return f"<Service {days}>"

    def available_at(self, date):
        return date not in self.removed and (date in self.added or self.regulary_available_at(date))

    def regulary_available_at(self, date):
        return self.start <= date <= self.end and self.available_weekdays[date.weekday()]

    @staticmethod
    def fromCsv(calendarCsv, calendarDatesCsv):
        reader = csv.DictReader(calendarCsv)

        servicesById = {}
        for row in reader:
            start = datetime.datetime.strptime(row["start_date"], "%Y%m%d").date()
            end = datetime.datetime.strptime(row["end_date"], "%Y%m%d").date()
            available_weekdays = (row[day] == "1" for day in ("monday", "tuesday", "wednesday", "thursday", "friday", "saturday", "sunday"))
            service = Service(start, end, tuple(available_weekdays))
            servicesById[row["service_id"]] = service

        reader = csv.DictReader(calendarDatesCsv)
        for row in reader:
            service = servicesById[row["service_id"]]
            date = datetime.datetime.strptime(row["date"], "%Y%m%d").date()
            if row["exception_type"] == "1":
                service.added.add(date)
            else:
                service.removed.add(date)

        return (list(servicesById.values()), servicesById)

class Agency(object):
    def __init__(self, name):
        self.name = name
        self.lines = []

    def __repr__(self):
        return f"<Agency '{self.name}'>"

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
    def __init__(self, name, lat, lon, parent):
        self.name = name
        self.lat = lat
        self.lon = lon
        self.parent = parent

    def __repr__(self):
        return f"<Location '{self.name}' | {self.lat} {self.lon}>"

    @property
    def station(self):
        if self.parent:
            return self.parent
        else:
            return self

    @staticmethod
    def fromCsv(locationCsv):
        reader = csv.DictReader(locationCsv)

        locationsById = {}
        queued = []
        for row in reader:
            if row["parent_station"]:
                queued.append(row)
            else:
                Location.parseRow(row, locationsById)

        for row in queued:
            Location.parseRow(row, locationsById)

        return (list(locationsById.values()), locationsById)

    @staticmethod
    def parseRow(row, locationsById):
        parent = None
        if row["parent_station"]:
            parent = locationsById[row["parent_station"]]
        location = Location(row["stop_name"], float(row["stop_lat"]), float(row["stop_lon"]), parent)
        locationsById[row["stop_id"]] = location
        return location

class Line(object):
    def __init__(self, agency, name, type):
        self.agency = agency
        self.name = name
        self.type = type
        self.routes = []

    def __eq__(self, other):
        return self.agency == other.agency and self.name == other.name and self.type == other.type

    def __hash__(self):
        return hash((self.agency, self.name, self.type))

    def __repr__(self):
        return f"<Line {self.name}>"

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

        lines = {}
        linesById = {}
        for row in reader:
            line = Line(agencies[row["agency_id"]], row["route_short_name"], int(row["route_type"]))

            if line in lines:
                line = lines[line]
            else:
                lines[line] = line
                line.agency.lines.append(line)

            linesById[row["route_id"]] = line

        return (list(lines.values()), linesById)

    @staticmethod
    def addColorsFromCsv(colorCsv, lines):
        reader = csv.DictReader(colorCsv, delimiter=";")

        for row in reader:
            for line in lines:
                if line.name == row["Linie"] and line.should_have_color:
                    line.color = row["Hex"]

class Route(object):
    def __init__(self, stations):
        self.stations = stations
        self.trips = []

    def __repr__(self):
        return f"<Route '{self.stations[0].name}' - '{self.stations[-1].name}>"

class Trip(object):
    def __init__(self, reversedDirection, times, service):
        self.reversedDirection = reversedDirection
        self.times = times
        self.service = service

    @staticmethod
    def fromCsv(tripCsv, stopCsv, locationsById, linesById, servicesById):
        reader = csv.DictReader(tripCsv)

        tripsById = {}
        for row in reader:
            tripsById[row["trip_id"]] = {
                "service": servicesById[row["service_id"]],
                "line": linesById[row["route_id"]],
                "stops": [],
            }

        reader = csv.DictReader(stopCsv)
        for row in reader:
            arrival = parse_timedelta(row["arrival_time"])
            departure = parse_timedelta(row["departure_time"])
            tripsById[row["trip_id"]]["stops"].append({
                "time": StopTime(arrival, departure),
                "station": locationsById[row["stop_id"]].station,
                "order": int(row["stop_sequence"]),
            })

        for trip in tripsById.values():
            trip["stops"].sort(key=itemgetter("order"))

            stations = [stop["station"] for stop in trip["stops"]]
            reversedStations = list(reversed(stations))
            reversedDirection = False

            lineRoutes = trip["line"].routes
            for route in lineRoutes:
                if route.stations == stations:
                    break
                if route.stations == reversedStations:
                    reversedDirection = True
                    break
            else:
                route = Route(stations)
                lineRoutes.append(route)

            times = [stop["time"] for stop in trip["stops"]]
            trip = Trip(reversedDirection, times, trip["service"])
            route.trips.append(trip)

class StopTime(object):
    def __init__(self, arrival, departure):
        self.arrival = arrival
        self.departure = departure

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
    date = datetime.date(2019, 5, 7)
    stations = set()
    lines = []

    for agency in dataset.agencies:
        if agency.name == "S-Bahn Berlin GmbH":
            for line in agency.lines:
                if line.is_suburban_railway:
                    route = max(line.routes, key=lambda route: len(route.trips))
                    stations |= set(route.stations)
                    lines.append((line, route))

    stations = list(stations)
    data = {
        "lines": [],
        "stations": [],
    }
    for line, route in lines:
        data["lines"].append({
            "name": line.name,
            "color": line.color,
            "stops": [stations.index(station) for station in route.stations]
        })

    for station in stations:
        data["stations"].append({
            "name": station.name,
            "x": round(2000 * (station.lon - 13.5), 3),
            "y": round(-4000 * (station.lat - 52.52), 3),
        })

    print("Exporting...")
    with open("vbb.json", "w") as jsonfile:
        json.dump(data, jsonfile)
