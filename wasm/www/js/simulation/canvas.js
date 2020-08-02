import {View} from "../../wasm/tiny_transport_wasm.js";
import {Canvas} from "../base/canvas.js";
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
            this.setUpFramebuffer(),
        ]);
    }

    setUpFramebuffer() {
        this.renderbuffers = {
            color: this.gl.createRenderbuffer(),
            id: this.gl.createRenderbuffer(),
        };

        this.resizeRenderbuffers();

        this.framebuffer = this.gl.createFramebuffer();
        this.gl.bindFramebuffer(this.gl.FRAMEBUFFER, this.framebuffer);

        this.gl.framebufferRenderbuffer(this.gl.FRAMEBUFFER, this.gl.COLOR_ATTACHMENT0, this.gl.RENDERBUFFER, this.renderbuffers.color);
        this.gl.framebufferRenderbuffer(this.gl.FRAMEBUFFER, this.gl.COLOR_ATTACHMENT1, this.gl.RENDERBUFFER, this.renderbuffers.id);

        if (this.gl.checkFramebufferStatus(this.gl.FRAMEBUFFER) !== this.gl.FRAMEBUFFER_COMPLETE) {
            throw new Error("incomplete framebuffer");
        }
    }

    resizeRenderbuffers() {
        this.gl.bindRenderbuffer(this.gl.RENDERBUFFER, this.renderbuffers.color);
        this.gl.renderbufferStorage(this.gl.RENDERBUFFER, this.gl.RGBA8, this.canvas.width, this.canvas.height);

        this.gl.bindRenderbuffer(this.gl.RENDERBUFFER, this.renderbuffers.id);
        this.gl.renderbufferStorage(this.gl.RENDERBUFFER, this.gl.R16UI, this.canvas.width, this.canvas.height);
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
        if (this.renderbuffers) {
            this.resizeRenderbuffers();
        }
    }

    addControlListeners() {
        this.canvas.addEventListener("mousemove", event => {
            const x = event.clientX - this.canvas.offsetLeft;
            const y = this.canvas.height - (event.clientY - this.canvas.offsetTop);
            this.pickObject(x, y);
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

    pickObject(x, y) {
        const data = new Uint32Array(4);

        this.gl.bindFramebuffer(this.gl.READ_FRAMEBUFFER, this.framebuffer);
        this.gl.readBuffer(this.gl.COLOR_ATTACHMENT1);
        this.gl.readPixels(x, y, 1, 1, this.gl.RGBA_INTEGER, this.gl.UNSIGNED_INT, data, 0);
        this.gl.bindFramebuffer(this.gl.READ_FRAMEBUFFER, null);

        this.updateTooltip(data[0]);
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
        if (!this.renderbuffers) {
            return;
        }

        this.gl.clearBufferfv(this.gl.COLOR, 0, [0.9, 0.95, 0.95, 1.0]);
        this.gl.clearBufferuiv(this.gl.COLOR, 1, [0, 0, 0, 0]);
    }

    draw() {
        this.gl.bindFramebuffer(this.gl.FRAMEBUFFER, this.framebuffer);
        super.draw();
        this.gl.drawBuffers([
            this.gl.COLOR_ATTACHMENT0,
        ]);
        this.renderer.line.run();
        this.renderer.train.run();
        this.gl.drawBuffers([
            this.gl.COLOR_ATTACHMENT0,
            this.gl.COLOR_ATTACHMENT1,
        ]);
        this.renderer.station.run();
        this.gl.bindFramebuffer(this.gl.READ_FRAMEBUFFER, this.framebuffer);
        this.gl.bindFramebuffer(this.gl.DRAW_FRAMEBUFFER, null);
        this.gl.readBuffer(this.gl.COLOR_ATTACHMENT0);
        this.gl.blitFramebuffer(
            0, 0, this.canvas.width, this.canvas.height,
            0, 0, this.canvas.width, this.canvas.height,
            this.gl.COLOR_BUFFER_BIT, this.gl.NEAREST,
        );
        this.gl.bindFramebuffer(this.gl.FRAMEBUFFER, null);
    }
}
