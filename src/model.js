const vec2 = glMatrix.vec2;

export class Model {
    async setUp(json) {
        const data = JSON.parse(await json);
        this.stations = data.locations.map(location => {
            return new Station(location.lat, location.lon);
        });
        this.lines = data.routes.map(route => {
            const stops = route.stops.map(index => this.stations[index]);
            return new Line(route.color, stops);
        });
    }
}

class Station {
    constructor(lat, lon) {
        this.lat = lat;
        this.lon = lon;
    }

    get vertices() {
        return [this.lon, this.lat];
    }
}

class Line {
    constructor(color, stops) {
        this.color = color;
        this.stops = stops;
        this.segments = [];

        for (let i = 0; i < this.stops.length - 1; i++) {
            const segment = new LineSegment(this.stops[i], this.stops[i + 1]);
            this.segments.push(segment);

            if (i > 0) {
                segment.connectToPreceding(this.segments[i - 1]);
            }
        }
    }

    get vertices() {
        let vertices = this.segments.reduce((vertices, segment) => {
            return vertices.concat(segment.startVertices);
        }, []);
        return vertices.concat(this.segments[this.segments.length - 1].endVertices);
    }
}

class LineSegment {
    constructor(start, end) {
        this.start = start;
        this.end = end;

        const startVector = vec2.fromValues(start.lon, start.lat);
        const endVector = vec2.fromValues(end.lon, end.lat);
        this.direction = vec2.subtract(vec2.create(), startVector, endVector);
        this.direction[1] *= 2;
    }

    connectToPreceding(preceding) {
        this.preceding = preceding;
        preceding.following = this;
    }

    orthogonalConnection(point, segment) {
        let orthogonal = vec2.fromValues(segment[1], -segment[0]);
        vec2.normalize(orthogonal, orthogonal);
        return [
            point.lon + 0.002 * orthogonal[0], point.lat + 0.001 * orthogonal[1],
            point.lon - 0.002 * orthogonal[0], point.lat - 0.001 * orthogonal[1],
        ];
    }

    miterConnection(point, preceding, following) {
        const orthogonal = vec2.fromValues(preceding[1], -preceding[0]);
        const scaledPreceding = vec2.scale(vec2.create(), preceding, -vec2.length(following));
        const scaledFollowing = vec2.scale(vec2.create(), following, vec2.length(preceding));
        let miter = vec2.add(vec2.create(), scaledPreceding, scaledFollowing);
        miter = vec2.scale(miter, miter, 1 / vec2.dot(orthogonal, following));
        return [
            point.lon + 0.002 * miter[0], point.lat + 0.001 * miter[1],
            point.lon - 0.002 * miter[0], point.lat - 0.001 * miter[1],
        ];
    }

    get startVertices() {
        if (!this.preceding) {
            return this.orthogonalConnection(this.start, this.direction);
        } else {
            return this.miterConnection(this.start, this.preceding.direction, this.direction);
        }
    }

    get endVertices() {
        if (!this.following) {
            return this.orthogonalConnection(this.end, this.direction);
        } else {
            return this.miterConnection(this.end, this.direction, this.following.direction);
        }
    }
}
