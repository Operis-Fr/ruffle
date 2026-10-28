package {
import flash.display.Sprite;
import flash.text.engine.*;
import flash.geom.Rectangle;

public class Test extends Sprite {
    public function Test() {
        var fmt:ElementFormat = new ElementFormat(new FontDescription("Noto Sans"), 12);
        dump("Lorem ipsum dolor sit amet, consectetur\nadipiscing", fmt, 100, false);
        dump("Supercalifragilisticexpialidocious", fmt, 50, false);
        dump("abc", fmt, 0, false);
        dump("abc", fmt, 0, true);
        dump("A😀B c", fmt, 1000, false);
        var m:FontMetrics = fmt.getFontMetrics();
        trace("metrics emBox=" + m.emBox + " u=" + m.underlineOffset + " s=" + m.strikethroughOffset);
    }

    private function dump(text:String, fmt:ElementFormat, width:Number, fit:Boolean):void {
        var block:TextBlock = new TextBlock(new TextElement(text, fmt));
        trace("== " + text + " width=" + width + " fit=" + fit);
        var line:TextLine = block.createTextLine(null, width, 0, fit);
        trace(" result=" + block.textLineCreationResult);
        while (line) {
            trace(" line begin=" + line.textBlockBeginIndex + " len=" + line.rawTextLength
                + " [" + text.substr(line.textBlockBeginIndex, line.rawTextLength) + "]"
                + " tw=" + line.textWidth + " utw=" + line.unjustifiedTextWidth + " th=" + line.textHeight
                + " asc=" + line.ascent + " desc=" + line.descent
                + " w=" + line.width + " h=" + line.height + " bounds=" + line.getBounds(line)
                + " atoms=" + line.atomCount);
            for (var i:int = 0; i < line.atomCount; i++) {
                trace("   atom " + i + " " + line.getAtomTextBlockBeginIndex(i) + "-" + line.getAtomTextBlockEndIndex(i)
                    + " b=" + line.getAtomBounds(i) + " c=" + line.getAtomCenter(i) + " wb=" + line.getAtomWordBoundaryOnLeft(i));
            }
            line = block.createTextLine(line, width, 0, fit);
            trace(" result=" + block.textLineCreationResult);
        }
        trace(" last=" + (block.lastLine != null) + " first=" + (block.firstLine != null));
    }
}
}
