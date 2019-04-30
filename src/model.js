const vec2 = glMatrix.vec2;

export class Model {
    async setUp(json) {
        const data = JSON.parse(await json);
        this.stations = data.stations.map(location => {
            return new Station(location.lat, location.lon);
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
        this.tracks = {};
    }

    get key() {
        return `${this.lat},${this.lon}`;
    }

    nextAvailableTrackTo(station) {
        if (!this.tracks[station.key]) {
            this.tracks[station.key] = 0;
        }

        const track = this.tracks[station.key];
        this.tracks[station.key] += 1;
        return track;
    }

    get vertices() {
        return [this.lon, this.lat];
    }
}

class Line {
    constructor(color, stops) {
        this.color = color;
        this.stops = stops.map(station => new LineStop(station));

        for (let i = 0; i < this.stops.length - 1; i++) {
            const start = this.stops[i];
            const end = this.stops[i + 1];
            let segment = vec2.subtract(vec2.create(), start.stationVector, end.stationVector);
            segment[1] *= 2;
            start.addFollowing(end.station, segment);
            end.addPreceding(start.station, segment);
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
        this.stationVector = vec2.fromValues(this.station.lon, this.station.lat);
    }

    addPreceding(preceding, precedingDirection) {
        this.precedingTrack = this.station.nextAvailableTrackTo(preceding);
        this.precedingDirection = precedingDirection;
    }

    addFollowing(following, followingDirection) {
        this.followingTrack = this.station.nextAvailableTrackTo(following);
        this.followingDirection = followingDirection;
    }

    connection(orientation, track) {
        vec2.multiply(orientation, orientation, vec2.fromValues(0.0008, 0.0004));
        const trackOffset = Math.ceil(track / 2) * (-1) ** track;
        return [
            ...vec2.add(vec2.create(), this.stationVector, vec2.scale(vec2.create(), orientation, trackOffset + 0.5)),
            ...vec2.add(vec2.create(), this.stationVector, vec2.scale(vec2.create(), orientation, trackOffset - 0.5)),
        ];
    }

    orthogonalConnection(segment, track) {
        let orthogonal = vec2.fromValues(segment[1], -segment[0]);
        vec2.normalize(orthogonal, orthogonal);
        return this.connection(orthogonal, track);
    }

    miterConnection(track) {
        const orthogonal = vec2.fromValues(this.precedingDirection[1], -this.precedingDirection[0]);
        const scaledPreceding = vec2.scale(vec2.create(), this.precedingDirection, -vec2.length(this.followingDirection));
        const scaledFollowing = vec2.scale(vec2.create(), this.followingDirection, vec2.length(this.precedingDirection));
        let miter = vec2.add(vec2.create(), scaledPreceding, scaledFollowing);
        miter = vec2.scale(miter, miter, 1 / vec2.dot(orthogonal, this.followingDirection));
        return this.connection(miter, track);
    }

    get vertices() {
        if (!this.precedingDirection) {
            return this.orthogonalConnection(this.followingDirection, this.followingTrack);
        } else if (!this.followingDirection) {
            return this.orthogonalConnection(this.precedingDirection, this.precedingTrack);
        } else {
            let vertices = this.miterConnection(this.precedingTrack);
            if (this.precedingTrack != this.followingTrack) {
                vertices = vertices.concat(this.miterConnection(this.followingTrack));
            }
            return vertices;
        }
    }
}
