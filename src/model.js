const vec2 = glMatrix.vec2;
const mat2 = glMatrix.mat2;

export class Model {
    async setUp(json) {
        const data = JSON.parse(await json);
        this.stations = data.stations.map(station => {
            return new Station(station.name, station.x, station.y);
        });
        this.trains = [];
        this.lines = data.lines.map(line => {
            const stops = line.stops.map(index => this.stations[index]);
            return new Line(line.name, line.color, stops);
        });
        this.trains = data.lines.reduce((allTrains, line, index) => {
            const lineTrains = line.trips.map(trip => {
                return new Train(this.lines[index], trip.direction, trip.arrivals, trip.departures);
            });
            return allTrains.concat(lineTrains);
        }, []);
    }

    findEntity(point) {
        return this.stations.find(station => station.contains(point));
    }
}

class Station {
    constructor(name, x, y) {
        this.name = name;
        this.x = x;
        this.y = y;
        this.position = vec2.fromValues(this.x, this.y);
        this.trackBundles = {};
    }

    get key() {
        return `${this.x},${this.y}`;
    }

    fetchTrackTo(station, color) {
        let direction = vec2.subtract(vec2.create(), station.position, this.position);
        let bundle = this.trackBundles[station.key];
        if (!bundle) {
            bundle = new TrackBundle(direction);
            this.trackBundles[station.key] = bundle;
            station.trackBundles[this.key] = bundle;
        }

        return bundle.fetchTrack(direction, color);
    }

    contains(point) {
        const difference = vec2.subtract(vec2.create(), this.position, point);
        return vec2.length(difference) < 5.0;
    }

    get vertices() {
        return this.position;
    }
}

class TrackBundle {
    constructor(direction) {
        this.orthogonal = vec2.fromValues(direction[1], -direction[0]);
        this.tracks = {};
        this.count = 0;
    }

    fetchTrack(direction, color) {
        if (!this.tracks[color]) {
            this.tracks[color] = this.buildTrack(direction);
        }

        return this.tracks[color];
    }

    buildTrack(direction) {
        const track = new Track(direction, this.orthogonal, this.count);
        this.count += 1;
        return track;
    }
}

class Track {
    constructor(direction, orthogonal, number) {
        this.direction = direction;
        this.orthogonal = orthogonal;
        this.number = number;
        const normalized = vec2.normalize(vec2.create(), this.direction);
        this.orientation = mat2.fromValues(normalized[0], normalized[1], normalized[1], -normalized[0]);
    }

    reverse() {
        const reversedDirection = vec2.negate(vec2.create(), this.direction);
        return new Track(reversedDirection, this.orthogonal, this.number);
    }

    get offset() {
        return Math.ceil(this.number / 2) * (-1) ** this.number;
    }
}

class Line {
    constructor(name, color, stops) {
        this.name = name;
        this.color = color;
        this.stops = stops.map(station => new LineStop(station));

        for (let i = 0; i < this.stops.length - 1; i++) {
            const start = this.stops[i];
            const end = this.stops[i + 1];
            const track = start.station.fetchTrackTo(end.station, this.color);
            start.followingTrack = track;
            end.precedingTrack = track.reverse();
        }

        const colorAsInteger = parseInt(this.color.substring(1), 16);
        this.colorComponents = [colorAsInteger >> 16, colorAsInteger >> 8, colorAsInteger]
            .map(component => (component & 255) / 255);
    }

    get vertices() {
        return this.stops.reduce((vertices, stop) => {
            return vertices.concat(stop.vertices);
        }, []);
    }
}

class LineStop {
    constructor(station) {
        this.station = station;
    }

    connection(orientation, track) {
        return [
            ...vec2.scaleAndAdd(vec2.create(), this.station.position, orientation, track.offset + 0.5),
            ...vec2.scaleAndAdd(vec2.create(), this.station.position, orientation, track.offset - 0.5),
        ];
    }

    orthogonalConnection(track) {
        const orthogonal = vec2.normalize(vec2.create(), track.orthogonal);
        return this.connection(orthogonal, track);
    }

    miterConnection() {
        const scaledPreceding = vec2.scale(vec2.create(), this.precedingTrack.direction, vec2.length(this.followingTrack.direction));
        const scaledFollowing = vec2.scale(vec2.create(), this.followingTrack.direction, vec2.length(this.precedingTrack.direction));
        const span = vec2.add(vec2.create(), scaledPreceding, scaledFollowing);
        let miter = vec2.scale(vec2.create(), span, 1 / vec2.dot(this.precedingTrack.orthogonal, this.followingTrack.direction));
        let vertices = this.connection(miter, this.precedingTrack);

        if (this.precedingTrack.number != this.followingTrack.number) {
            miter = vec2.scale(vec2.create(), span, 1 / vec2.dot(this.followingTrack.orthogonal, this.precedingTrack.direction));
            vertices = vertices.concat(this.connection(miter, this.followingTrack));
        }

        return vertices;
    }

    miterOrientation(a, b) {
        const scaledPreceding = vec2.scale(vec2.create(), this.precedingTrack.direction, vec2.length(this.followingTrack.direction));
        const scaledFollowing = vec2.scale(vec2.create(), this.followingTrack.direction, vec2.length(this.precedingTrack.direction));
        const span = vec2.add(vec2.create(), scaledPreceding, scaledFollowing);
        return vec2.scale(vec2.create(), span, 1 / vec2.dot(a.orthogonal, b.direction));
    }

    miterOrientation(a, b) {
        const scaledPreceding = vec2.scale(vec2.create(), this.precedingTrack.direction, vec2.length(this.followingTrack.direction));
        const scaledFollowing = vec2.scale(vec2.create(), this.followingTrack.direction, vec2.length(this.precedingTrack.direction));
        const span = vec2.add(vec2.create(), scaledPreceding, scaledFollowing);
        return vec2.scale(vec2.create(), span, 1 / vec2.dot(a.orthogonal, b.direction));
    }

    get precedingOrientation() {
        if (this.followingTrack) {
            return this.miterOrientation(this.precedingTrack, this.followingTrack);
        } else {
            return vec2.normalize(vec2.create(), this.precedingTrack.orthogonal);
        }
    }

    get followingOrientation() {
        if (this.precedingTrack) {
            return this.miterOrientation(this.followingTrack, this.precedingTrack);
        } else {
            return vec2.normalize(vec2.create(), this.followingTrack.orthogonal);
        }
    }

    get precedingPosition() {
        return vec2.add(vec2.create(), this.station.position, vec2.scale(vec2.create(), this.precedingOrientation, this.precedingTrack.offset));
    }

    get followingPosition() {
        return vec2.add(vec2.create(), this.station.position, vec2.scale(vec2.create(), this.followingOrientation, this.followingTrack.offset));
    }

    get vertices() {
        if (this.precedingTrack && this.followingTrack) {
            return this.miterConnection();
        } else if (this.precedingTrack) {
            return this.orthogonalConnection(this.precedingTrack);
        } else if (this.followingTrack) {
            return this.orthogonalConnection(this.followingTrack);
        }
    }
}

class Train {
    constructor(line, direction, arrivals, departures) {
        this.line = line;
        this.direction = direction;
        this.arrivals = arrivals;
        this.departures = departures;
        this.current = 0;
    }

    update(time) {
        while (time > this.arrivals[this.current]) {
            this.current += 1;

            if (this.current <= this.line.stops.length - 1) {
                this.travelTime = this.arrivals[this.current] - this.departures[this.current - 1];
                if (this.direction === "upstream") {
                    this.departurePosition = this.line.stops[this.current - 1].followingPosition;
                    this.track = this.line.stops[this.current - 1].followingTrack;
                } else {
                    this.departurePosition = this.line.stops[this.line.stops.length - this.current].precedingPosition;
                    this.track = this.line.stops[this.line.stops.length - this.current].precedingTrack;
                }
            }
        }

        this.travelled = Math.max(time - this.departures[this.current - 1], 0) / this.travelTime;
    }

    get isActive() {
        return this.current > 0 && this.current < this.line.stops.length;
    }

    get vertices() {
        const position = vec2.scaleAndAdd(vec2.create(), this.departurePosition, this.track.direction, this.travelled);

        const rightFront = vec2.add(vec2.create(), position, vec2.transformMat2(vec2.create(), vec2.fromValues(4.5, 3), this.track.orientation));
        const leftFront = vec2.add(vec2.create(), position, vec2.transformMat2(vec2.create(), vec2.fromValues(-4.5, 3), this.track.orientation));
        const rightBack = vec2.add(vec2.create(), position, vec2.transformMat2(vec2.create(), vec2.fromValues(4.5, -3), this.track.orientation));
        const leftBack = vec2.add(vec2.create(), position, vec2.transformMat2(vec2.create(), vec2.fromValues(-4.5, -3), this.track.orientation));
        return [
            ...leftBack, ...leftFront, ...rightBack,
            ...rightFront, ...rightBack, ...leftFront,
        ];
    }

    get colors() {
        return [
            ...this.line.colorComponents, ...this.line.colorComponents, ...this.line.colorComponents,
            ...this.line.colorComponents, ...this.line.colorComponents, ...this.line.colorComponents,
        ];
    }
}
