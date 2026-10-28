package flash.desktop {
    import __ruffle__.stub_method;


    public class Clipboard {
        private static var _generalClipboard = new Clipboard();

        public static function get generalClipboard():Clipboard {
            return Clipboard._generalClipboard;
        }

        function Clipboard() {
            // TODO: This should only be callable in AIR
        }

        // Data set by the movie for formats other than text, along with the
        // system clipboard text at the time it was set. That data is only
        // considered valid while the system clipboard still holds that text,
        // so that the movie doesn't paste stale data after the user copied
        // something from another application.
        private var _data:Object = {};
        private var _textWhenSet:String = null;

        private static native function getSystemText():String;
        private static native function setSystemText(text:String):void;

        private function nonTextDataValid():Boolean {
            return this._textWhenSet != null && this._textWhenSet == getSystemText();
        }

        public function get formats():Array {
            var result:Array = [];
            if (getSystemText() != "") {
                result.push(ClipboardFormats.TEXT_FORMAT);
            }
            if (this.nonTextDataValid()) {
                for (var format:String in this._data) {
                    result.push(format);
                }
            }
            return result;
        }

        public function clear():void {
            this._data = {};
            this._textWhenSet = null;
        }

        public function clearData(format:String):void {
            delete this._data[format];
        }

        public function getData(
            format:String,
            transferMode:String = ClipboardTransferMode.ORIGINAL_PREFERRED
        ):Object {
            if (format == ClipboardFormats.TEXT_FORMAT) {
                var text:String = getSystemText();
                return text == "" ? null : text;
            }
            if (this.nonTextDataValid() && this._data.hasOwnProperty(format)) {
                return this._data[format];
            }
            return null;
        }

        public function hasFormat(format:String):Boolean {
            if (format == ClipboardFormats.TEXT_FORMAT) {
                return getSystemText() != "";
            }
            return this.nonTextDataValid() && this._data.hasOwnProperty(format);
        }

        public function setData(format:String, data:Object, serializable:Boolean = true):Boolean {
            if (format == ClipboardFormats.TEXT_FORMAT) {
                setSystemText(String(data));
            } else if (format == ClipboardFormats.BITMAP_FORMAT
                || format == ClipboardFormats.FILE_LIST_FORMAT
                || format == ClipboardFormats.FILE_PROMISE_LIST_FORMAT) {
                stub_method("flash.desktop.Clipboard", "setData", "with non-text system format");
                return false;
            } else {
                // Custom (or HTML/RTF/URL) formats only live inside the player.
                this._data[format] = data;
            }
            this._textWhenSet = getSystemText();
            return true;
        }

        public function setDataHandler(format:String, handler:Function, serializable:Boolean = true):Boolean {
            stub_method("flash.desktop.Clipboard", "setDataHandler");
            return false;
        }
    }
}
