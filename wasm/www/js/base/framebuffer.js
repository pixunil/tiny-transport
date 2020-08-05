export class Framebuffer {
    constructor(gl, attachments, width, height, samples = 0) {
        this.gl = gl;
        this.attachments = new Map();
        this.samples = samples;
        for (const [i, attachment] of attachments.entries()) {
            this.attachments.set(attachment.name, {
                renderbuffer: this.gl.createRenderbuffer(),
                index: i,
                attachment: this.gl.COLOR_ATTACHMENT0 + i,
                type: attachment.type,
            });
        }
        this.resize(width, height);

        this.framebuffer = this.gl.createFramebuffer();
        this.gl.bindFramebuffer(this.gl.FRAMEBUFFER, this.framebuffer);
        for (const attachment of this.attachments.values()) {
            this.gl.framebufferRenderbuffer(this.gl.FRAMEBUFFER, attachment.attachment, this.gl.RENDERBUFFER, attachment.renderbuffer);
        }

        if (this.gl.checkFramebufferStatus(this.gl.FRAMEBUFFER) !== this.gl.FRAMEBUFFER_COMPLETE) {
            throw new Error("incomplete framebuffer");
        }
    }

    resize(width, height) {
        this.size = [width, height];
        for (const attachment of this.attachments.values()) {
            this.gl.bindRenderbuffer(this.gl.RENDERBUFFER, attachment.renderbuffer);
            this.gl.renderbufferStorageMultisample(this.gl.RENDERBUFFER, this.samples, attachment.type, width, height);
        }
    }

    bind(drawAttachmentNames) {
        this.gl.bindFramebuffer(this.gl.DRAW_FRAMEBUFFER, this.framebuffer);
        this.gl.drawBuffers(drawAttachmentNames.map(name => this.attachments.get(name).attachment));
    }

    clear(attachmentName, clearValue) {
        const attachment = this.attachments.get(attachmentName);
        let clearFunction;
        if (attachment.type === this.gl.RGB8) {
            clearFunction = this.gl.clearBufferfv;
        } else if (attachment.type === this.gl.R16UI) {
            clearFunction = this.gl.clearBufferuiv;
        }
        clearFunction.call(this.gl, this.gl.COLOR, attachment.index, clearValue);
    }

    blit(framebuffer, readAttachmentName, drawAttachmentNames) {
        this.gl.bindFramebuffer(this.gl.READ_FRAMEBUFFER, this.framebuffer);
        this.gl.bindFramebuffer(this.gl.DRAW_FRAMEBUFFER, framebuffer?.framebuffer);
        this.gl.readBuffer(this.attachments.get(readAttachmentName).attachment);
        if (framebuffer) {
            this.gl.drawBuffers(drawAttachmentNames.map(name => this.attachments.get(name).attachment));
        }
        this.gl.blitFramebuffer(
            0, 0, this.size[0], this.size[1],
            0, 0, this.size[0], this.size[1],
            this.gl.COLOR_BUFFER_BIT, this.gl.NEAREST,
        );
    }

    pick(attachmentName, x, y) {
        let data = new Uint32Array(4);
        this.gl.bindFramebuffer(this.gl.READ_FRAMEBUFFER, this.framebuffer);
        this.gl.readBuffer(this.attachments.get(attachmentName).attachment);
        this.gl.readPixels(x, y, 1, 1, this.gl.RGBA_INTEGER, this.gl.UNSIGNED_INT, data, 0);
        this.gl.bindFramebuffer(this.gl.READ_FRAMEBUFFER, null);
        return data[0];
    }
}
