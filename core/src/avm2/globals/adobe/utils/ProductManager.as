package adobe.utils {
    import __ruffle__.stub_getter;
    import __ruffle__.stub_method;

    import flash.events.EventDispatcher;

    // Used to download/launch Adobe add-ons (e.g. "adobeflashenabler" on Chrome's Pepper Flash).
    // There is nothing to download in Ruffle: report the product as installed so that content
    // skips its download path.
    public class ProductManager extends EventDispatcher {
        private var _name:String;

        public function ProductManager(name:String) {
            this._name = name;
        }

        public function get installed():Boolean {
            stub_getter("adobe.utils.ProductManager", "installed");
            return true;
        }

        public function get installedVersion():String {
            stub_getter("adobe.utils.ProductManager", "installedVersion");
            return "";
        }

        public function get running():Boolean {
            stub_getter("adobe.utils.ProductManager", "running");
            return false;
        }

        public function download(caption:String = null, fileName:String = null, pathElements:Array = null):Boolean {
            stub_method("adobe.utils.ProductManager", "download");
            return false;
        }

        public function launch(parameters:String = null):Boolean {
            stub_method("adobe.utils.ProductManager", "launch");
            return false;
        }
    }
}
