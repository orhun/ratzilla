export class RatzillaCanvas {
    constructor() {
        this.bold = false;
        this.italic = false;
    }

    create_canvas_in_element(parent, font_str, backgroundColor) {
        this.parent = document.getElementById(parent);
        if (this.parent == null) {
            this.parent = document.body;
        }
        // Uses input hack from https://github.com/emilk/egui/blob/fdcaff8465eac8db8cc1ebbcbb9b97e0791a8363/crates/eframe/src/web/text_agent.rs#L18
        this.inputElement = document.createElement("input");
        this.inputElement.autofocus = true;
        this.inputElement.type = "text";
        this.inputElement.autocapitalize = "off";
        this.inputElement.style.backgroundColor = "transparent";
        this.inputElement.style.border = "none";
        this.inputElement.style.outline = "none";
        this.inputElement.style.width = "1px";
        this.inputElement.style.height = "1px";
        this.inputElement.style.caretColor = "transparent";
        this.inputElement.style.position= "absolute";
        this.inputElement.style.top = "0";
        this.inputElement.style.left = "0";
        this.inputElement.addEventListener("input", (event) => {
            if (!event.isComposing) {
                this.inputElement.blur();
                this.inputElement.focus();
            }

            if (!(this.inputElement.value.length === 0) && !event.isComposing) {
                this.inputElement.value = ""
            }
        });
        this.canvas = document.createElement("canvas");
        this.canvas.tabIndex = 0;
        this.canvas.style.outline = "none";
        this.canvas.addEventListener("focus", () => {
            this.inputElement.focus();
        });
        this.parent.appendChild(this.inputElement);
        this.parent.appendChild(this.canvas);
        this.font_str = font_str;
        this.backgroundColor = `#${backgroundColor.toString(16).padStart(6, '0')}`;
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
        const ratio = window.devicePixelRatio;
        this.ctx = this.canvas.getContext("2d", {
            desynchronized: true
        });
        this.init_font();
        this.ctx.scale(ratio, ratio);
        this.ctx.fillStyle = this.backgroundColor;
        this.ctx.fillRect(0, 0, this.canvas.width, this.canvas.height);
    }

    init_font() {
        this.ctx.font = `${this.bold ? 'bold' : ''} ${this.italic ? 'italic' : ''} ${this.font_str}`;
    }

    get_input_element() {
        return this.textArea;
    }

    reinit_canvas() {
        const ratio = window.devicePixelRatio;
        let sourceW = Math.ceil(this.parent.clientWidth / this.cellWidth);
        let sourceH = Math.ceil(this.parent.clientHeight / this.cellHeight);

        let canvasW = sourceW * this.cellWidth;
        let canvasH = sourceH * this.cellHeight;

        if (this.canvas.width != canvasW * ratio || this.canvas.height != canvasH * ratio) {
            this.canvas.width = canvasW * ratio;
            this.canvas.height = canvasH * ratio;
            this.canvas.style.width = canvasW + "px";
            this.canvas.style.height = canvasH + "px";
            this.init_ctx();
        }
        
        return new Uint16Array([sourceW, sourceH]);
    }
}
