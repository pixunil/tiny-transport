import {View} from "../../wasm/tiny_transport_wasm.js";
import {Canvas} from "../base/canvas.js";
import {Framebuffer} from "../base/framebuffer.js";
import {LineRenderer} from "./line.js";
import {TrainRenderer} from "./train.js";
import {StationRenderer} from "./station.js";

export class SimulationCanvas extends Canvas {
    constructor(canvas) {
        super(canvas);
        this.addControlListeners();

        this.view = new View(0.08, this.canvas.width, this.canvas.height);
        this.view.viewProjection = this.view.calculateViewProjection();

        this.renderer = {
            line: new LineRenderer(this.view),
            train: new TrainRenderer(this.view),
            station: new StationRenderer(this.view),
        };
    }

    async setUp(assets) {
        await Promise.all([
            this.renderer.line.setUp(this.gl, assets.line),
            this.renderer.train.setUp(this.gl, assets.train),
            this.renderer.station.setUp(this.gl, assets.station),
            this.setUpFramebuffers(),
        ]);
    }

    setUpFramebuffers() {
        this.framebuffers = new Map();
        this.framebuffers.set("color", new Framebuffer(this.gl, [{
            name: "color",
            type: this.gl.RGB8,
        }], this.canvas.width, this.canvas.height, 16));
        this.framebuffers.set("color-id", new Framebuffer(this.gl, [{
            name: "color",
            type: this.gl.RGB8,
        }, {
            name: "id",
            type: this.gl.R16UI,
        }], this.canvas.width, this.canvas.height));
    }

    async setUpWithModel(model) {
        this.model = model;
        await Promise.all([
            this.renderer.line.fillBuffers(this.model),
            this.renderer.station.fillBuffers(this.model),
            this.renderer.train.generateTextures(this.model),
        ]);
    }

    update() {
        this.renderer.train.fillBuffers(this.model);
    }

    resizeCanvas() {
        super.resizeCanvas();

        if (this.view) {
            this.view.resize(this.canvas.width, this.canvas.height);
            this.view.viewProjection = this.view.calculateViewProjection();
        }
        if (this.framebuffers) {
            for (const framebuffer of this.framebuffers.values()) {
                framebuffer.resize(this.canvas.width, this.canvas.height);
            }
        }
    }

    addControlListeners() {
        this.canvas.addEventListener("mousemove", event => {
            if (this.framebuffers) {
                const x = event.clientX - this.canvas.offsetLeft;
                const y = this.canvas.height - (event.clientY - this.canvas.offsetTop);
                const id = this.framebuffers.get("color-id").pick("id", x, y);
                this.updateTooltip(id);
            }
            if (event.buttons) {
                this.view.scroll(event.movementX, event.movementY);
                this.view.viewProjection = this.view.calculateViewProjection();
            }
        });
        this.canvas.addEventListener("wheel", event => {
            const scaling = event.deltaY < 0 ? 11 / 10 : 10 / 11;
            this.view.zoom(scaling, event.clientX - this.canvas.offsetLeft, event.clientY - this.canvas.offsetTop);
            this.view.viewProjection = this.view.calculateViewProjection();
        });
    }

    updateTooltip(id) {
        if (this.model) {
            if (id === 0) {
                this.canvas.title = "";
            } else {
                this.canvas.title = this.model.stationName(id - 1);
            }
        }
    }

    clear() {
        this.gl.clearColor(0.9, 0.95, 0.95, 1.0);
        this.gl.clear(this.gl.COLOR_BUFFER_BIT);
    }

    draw() {
        super.draw();
        this.framebuffers.get("color").bind(["color"]);
        this.framebuffers.get("color").clear("color", [0.9, 0.95, 0.95, 1.0])
        this.renderer.line.run();
        this.renderer.train.run();
        this.framebuffers.get("color").blit(this.framebuffers.get("color-id"), "color", ["color"]);


        this.framebuffers.get("color-id").bind(["color", "id"]);
        this.framebuffers.get("color-id").clear("id", [0, 0, 0, 0]);
        this.renderer.station.run();
        this.framebuffers.get("color-id").blit(null, "color", []);
        this.gl.bindFramebuffer(this.gl.FRAMEBUFFER, null);
    }
}
