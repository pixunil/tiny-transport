import {Dataset, default as init} from "../wasm/gtfs_sim_wasm.js";
import {SimulationCanvas} from "./simulation/canvas.js";

class Controller {
    constructor() {
        this.wasm = init("wasm/gtfs_sim_wasm_bg.wasm");
    }

    async setUp() {
        await this.wasm;
        this.simulationCanvas = new SimulationCanvas(document.querySelector("canvas"));

        await Promise.all([
            this.setUpModel(),
            this.simulationCanvas.setUp(assets),
        ]);
        await this.simulationCanvas.setUpWithModel(this.model);

        this.milliseconds = performance.now();
        this.drawLoop(this.milliseconds);
    }

    async setUpModel() {
        this.model = Dataset.parse(await assets.data);
        this.model.update(14010);
    }

    drawLoop(milliseconds) {
        this.update(milliseconds);
        this.simulationCanvas.draw();
        requestAnimationFrame(milliseconds => this.drawLoop(milliseconds));
    }

    update(milliseconds) {
        const speed = parseInt(document.querySelector("input").value);
        const millisecondsPassed = milliseconds - this.milliseconds;
        this.milliseconds = milliseconds;
        const timePassed = Math.floor(millisecondsPassed * speed / 1000);
        this.model.update(timePassed);
        this.simulationCanvas.update();
    }
}

function fetchSource(url) {
    return fetch(url)
        .then(response => {
            if (!response.ok) {
                throw new Error(`Could not fetch ${url}`);
            }

            return response.text();
        });
}

function fetchBinary(url) {
    return fetch(url)
        .then(response => {
            if (!response.ok) {
                throw new Error(`Could not fetch ${url}`);
            }
            return response.arrayBuffer();
        })
        .then(arrayBuffer => new Uint8Array(arrayBuffer));
}

const assets = {
    line: {
        vertex: fetchSource("shader/line.vert.glsl"),
        fragment: fetchSource("shader/line.frag.glsl"),
    },
    train: {
        vertex: fetchSource("shader/train.vert.glsl"),
        fragment: fetchSource("shader/train.frag.glsl"),
    },
    station: {
        vertex: fetchSource("shader/station.vert.glsl"),
        fragment: fetchSource("shader/station.frag.glsl"),
    },
    data: fetchBinary("data.bin"),
};

const controller = new Controller();

addEventListener("load", () => {
    controller.setUp().catch(error => console.error(error));
});
