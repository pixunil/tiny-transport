import {Renderer} from "../base/renderer.js";

export class TrainRenderer extends Renderer {
    createBuffers() {
        this.buffers = {
            position: this.gl.createBuffer(),
            color: this.gl.createBuffer(),
            extent: this.gl.createBuffer(),
            lineNumber: this.gl.createBuffer(),
            side: this.gl.createBuffer(),
        }
    }

    generateTextures(model) {
        const names = model.lineNames().split("\n");
        this.lineNamesTextureGenerator = new LineNamesTextureGenerator(names, 0);

        this.texture = this.gl.createTexture();
        this.gl.bindTexture(this.gl.TEXTURE_2D, this.texture);
        this.lineNamesTextureGenerator.generateLineNameTexture(this.gl);
        const maxLevel = Math.log2(Math.max(this.lineNamesTextureGenerator.width, this.lineNamesTextureGenerator.height));
        for (let level = 1; level <= maxLevel; level++) {
            const lineNamesTextureGenerator = new LineNamesTextureGenerator(names, level);
            lineNamesTextureGenerator.generateLineNameTexture(this.gl);
        }

        this.gl.texParameteri(this.gl.TEXTURE_2D, this.gl.TEXTURE_MIN_FILTER, this.gl.LINEAR_MIPMAP_LINEAR);
        this.gl.texParameteri(this.gl.TEXTURE_2D, this.gl.TEXTURE_WRAP_S, this.gl.CLAMP_TO_EDGE);
        this.gl.texParameteri(this.gl.TEXTURE_2D, this.gl.TEXTURE_WRAP_T, this.gl.CLAMP_TO_EDGE);
    }

    fillBuffers(model) {
        this.count = model.trainCount();

        this.gl.bindBuffer(this.gl.ARRAY_BUFFER, this.buffers.position);
        this.gl.bufferData(this.gl.ARRAY_BUFFER, model.trainVertices(), this.gl.DYNAMIC_DRAW);

        this.gl.bindBuffer(this.gl.ARRAY_BUFFER, this.buffers.color);
        this.gl.bufferData(this.gl.ARRAY_BUFFER, model.trainColors(), this.gl.DYNAMIC_DRAW);

        this.gl.bindBuffer(this.gl.ARRAY_BUFFER, this.buffers.extent);
        this.gl.bufferData(this.gl.ARRAY_BUFFER, model.trainExtents(), this.gl.DYNAMIC_DRAW);

        this.gl.bindBuffer(this.gl.ARRAY_BUFFER, this.buffers.lineNumber);
        this.gl.bufferData(this.gl.ARRAY_BUFFER, model.trainLineNumbers(), this.gl.DYNAMIC_DRAW);

        this.gl.bindBuffer(this.gl.ARRAY_BUFFER, this.buffers.side);
        this.gl.bufferData(this.gl.ARRAY_BUFFER, model.trainSides(), this.gl.DYNAMIC_DRAW);
    }

    run() {
        this.gl.useProgram(this.programInfo.program);

        this.gl.uniformMatrix4fv(this.uniformLocations.modelView, false, this.view.viewProjection);
        this.gl.uniform2uiv(this.uniformLocations.lineNamesShape, this.lineNamesTextureGenerator.shape);
        this.gl.uniform1f(this.uniformLocations.lineNamesRatio, this.lineNamesTextureGenerator.ratio);

        this.gl.activeTexture(this.gl.TEXTURE0);
        this.gl.bindTexture(this.gl.TEXTURE_2D, this.texture);
        this.gl.uniform1i(this.uniformLocations.lineNames, 0);

        this.gl.bindBuffer(this.gl.ARRAY_BUFFER, this.buffers.position);
        this.gl.vertexAttribPointer(
            this.attributeLocations.position,
            2, this.gl.FLOAT,
            false, 0, 0);
        this.gl.enableVertexAttribArray(this.attributeLocations.position);

        this.gl.bindBuffer(this.gl.ARRAY_BUFFER, this.buffers.color);
        this.gl.vertexAttribPointer(
            this.attributeLocations.color,
            3, this.gl.FLOAT,
            false, 0, 0);
        this.gl.enableVertexAttribArray(this.attributeLocations.color);

        this.gl.bindBuffer(this.gl.ARRAY_BUFFER, this.buffers.extent);
        this.gl.vertexAttribPointer(
            this.attributeLocations.extent,
            2, this.gl.FLOAT,
            false, 0, 0);
        this.gl.enableVertexAttribArray(this.attributeLocations.extent);

        this.gl.bindBuffer(this.gl.ARRAY_BUFFER, this.buffers.lineNumber);
        this.gl.vertexAttribIPointer(
            this.attributeLocations.lineNumber,
            1, this.gl.UNSIGNED_SHORT, 0, 0);
        this.gl.enableVertexAttribArray(this.attributeLocations.lineNumber);

        this.gl.bindBuffer(this.gl.ARRAY_BUFFER, this.buffers.side);
        this.gl.vertexAttribPointer(
            this.attributeLocations.side,
            2, this.gl.UNSIGNED_BYTE,
            false, 0, 0);
        this.gl.enableVertexAttribArray(this.attributeLocations.side);

        this.gl.drawArrays(this.gl.TRIANGLES, 0, 6 * this.count);
    }
}

class LineNamesTextureGenerator {
    constructor(names, level) {
        this.names = names;
        this.level = level;
        const scale = 2 ** level;
        this.entryWidth = 256 / scale;
        this.entryHeight = 128 / scale;
        this.fontSize = 96 / scale;
    }

    get shape() {
        const exponent = Math.ceil(Math.log2(this.names.length));
        return [2 ** Math.floor(exponent / 2), 2 ** Math.ceil(exponent / 2)];
    }

    get width() {
        return this.shape[0] * this.entryWidth;
    }

    get height() {
        return this.shape[1] * this.entryHeight;
    }

    get ratio() {
        return this.entryHeight / this.entryWidth;
    }

    generateLineNameTexture(gl) {
        let canvas = document.createElement("canvas");
        let context = canvas.getContext("2d");
        canvas.width = this.width;
        canvas.height = this.height;

        context.fillStyle = "#ffffff";
        context.font = `${this.fontSize}px sans-serif`;
        for (let i = 0; i < this.names.length; i++) {
            const name = this.names[i];
            const measure = context.measureText(name);
            const x = (i % this.shape[0]) * this.entryWidth + (this.entryWidth - measure.width) / 2;
            const y = Math.floor(i / this.shape[0]) * this.entryHeight + this.fontSize;
            context.fillText(name, x, y);
        }

        gl.texImage2D(gl.TEXTURE_2D, this.level, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, canvas);
    }
}
