const vec2 = glMatrix.vec2;

export class Model {
    async setUp(json) {
        const data = JSON.parse(await json);
        this.stations = data.stations.map(station => {
            return new Station(station.lat, station.lon);
        });
        this.lines = data.lines.map(line => {
            const stops = line.stops.map(index => this.stations[index]);
            return new Line(line.color, stops);
        });
    }
}

class Station {
    constructor(lat, lon) {
        this.lat = lat;
        this.lon = lon;
        this.position = vec2.fromValues(this.lon, this.lat);
        this.trackBundles = {};
    }

    get key() {
        return `${this.lat},${this.lon}`;
    }

    buildTrackTo(station) {
        let direction = vec2.subtract(vec2.create(), this.position, station.position);
        direction[1] *= 2;
        let bundle = this.trackBundles[station.key];
        if (!bundle) {
            bundle = new TrackBundle(direction);
            this.trackBundles[station.key] = bundle;
            station.trackBundles[this.key] = bundle;
        }

        return bundle.buildTrack(direction);
    }

    get vertices() {
        return [this.lon, this.lat];
    }
}

class TrackBundle {
    constructor(direction) {
        this.orthogonal = vec2.fromValues(direction[1], -direction[0]);
        this.count = 0;
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
        this.sign = 1;
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
    constructor(color, stops) {
        this.color = color;
        this.stops = stops.map(station => new LineStop(station));

        for (let i = 0; i < this.stops.length - 1; i++) {
            const start = this.stops[i];
            const end = this.stops[i + 1];
            const track = start.station.buildTrackTo(end.station);
            start.followingTrack = track;
            end.precedingTrack = track.reverse();
        }
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
        vec2.multiply(orientation, orientation, vec2.fromValues(0.0008, 0.0004));
        return [
            ...vec2.add(vec2.create(), this.station.position, vec2.scale(vec2.create(), orientation, track.offset + 0.5)),
            ...vec2.add(vec2.create(), this.station.position, vec2.scale(vec2.create(), orientation, track.offset - 0.5)),
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
