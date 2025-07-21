export class RatzillaCanvas {
    constructor() {}

    create_canvas_in_element(parent, font_str) {
        this.parent = document.getElementById(parent);
        if (this.parent == null) {
            this.parent = document.body;
        }
        this.canvas = document.createElement("canvas");
        this.parent.appendChild(this.canvas);
        this.font_str = font_str;
        this.init_ctx();
    }

    // Very useful code from here https://github.com/ghostty-org/ghostty/blob/a88689ca754a6eb7dce6015b85ccb1416b5363d8/src/font/face/web_canvas.zig#L242
    measure_text() {
        // A character with max width, max height, and max bottom
        let metrics = this.ctx.measureText("█");
        if (metrics.actualBoundingBoxRight > 0) {
            this.cellWidth = Math.floor(metrics.actualBoundingBoxRight);
        } else {
            this.cellWidth = Math.floor(metrics.width);
        }
        this.cellHeight = Math.floor(metrics.actualBoundingBoxAscent + metrics.actualBoundingBoxDescent);
        this.cellBaseline = Math.floor(metrics.actualBoundingBoxDescent);
        this.underlinePos = Math.floor(this.cellHeight - 1.0);
        return new Uint16Array([this.cellWidth, this.cellHeight, this.cellBaseline, this.underlinePos]);
    }

    init_ctx() {
        this.ctx = this.canvas.getContext("2d", {
            alpha: true,
            desynchronized: true
        });
        this.ctx.font = this.font_str;
    }

    get_canvas() {
        return this.canvas;
    }

    reinit_canvas() {
        let sourceW = Math.ceil(this.parent.clientWidth / this.cellWidth);
        let sourceH = Math.ceil(this.parent.clientHeight / this.cellHeight);

        let canvasW = sourceW * this.cellWidth;
        let canvasH = sourceH * this.cellHeight;

        if (this.canvas.width != canvasW || this.canvas.height != canvasH) {
            let dummyCanvas = new OffscreenCanvas(this.canvas.width, this.canvas.height);
            let dummyCtx = dummyCanvas.getContext('2d', {
                alpha: true
            });

            dummyCtx.drawImage(this.canvas, 0, 0);

            this.canvas.width = canvasW;
            this.canvas.height = canvasH;
            this.init_ctx();

            this.ctx.drawImage(dummyCanvas,
                0, 0,
                canvasW, canvasH,
                0, 0,
                canvasW, canvasH,
            );
        }
        
        return new Uint16Array([sourceW, sourceH]);
    }
}
