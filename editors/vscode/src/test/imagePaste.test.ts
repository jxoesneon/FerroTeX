import * as assert from 'assert';
import * as path from 'path';
import { ImagePasteProvider } from '../imagePaste';

suite('ImagePasteProvider Test Suite', () => {
    const provider = new ImagePasteProvider();

    test('generateFilename: replaces timestamp', () => {
        const pattern = 'img-{timestamp}';
        const result = provider.generateFilename(pattern);
        assert.ok(result.startsWith('img-'));
        assert.ok(result.endsWith('.png'));
        // ISO string has many numbers, check length roughly
        assert.ok(result.length > 10);
    });

    test('generateFilename: replaces uuid', () => {
        const pattern = 'pic-{uuid}';
        const result = provider.generateFilename(pattern);
        assert.ok(result.startsWith('pic-'));
        assert.ok(!result.includes('{uuid}'));
    });

    test('generateFilename: ensures png extension', () => {
        const result = provider.generateFilename('test');
        assert.strictEqual(result, 'test.png');
    });

    test('resolveRelativePath: sibling directory', () => {
        // Doc: /project/main.tex
        // Img: /project/figures/img.png
        // Rel: figures/img.png
        // Note: We simulate paths. On windows path.sep is \, we need to ensure tests pass logic.
        const docPath = path.join('/project', 'main.tex');
        const imgPath = path.join('/project', 'figures', 'img.png');
        
        const relative = provider.resolveRelativePath(docPath, imgPath);
        assert.strictEqual(relative, 'figures/img.png');
    });

    test('resolveRelativePath: same directory', () => {
        const docPath = path.join('/project', 'main.tex');
        const imgPath = path.join('/project', 'img.png');
        
        const relative = provider.resolveRelativePath(docPath, imgPath);
        assert.strictEqual(relative, 'img.png');
    });

    test('resolveRelativePath: parent directory', () => {
        const docPath = path.join('/project', 'subdir', 'main.tex');
        const imgPath = path.join('/project', 'img.png');
        
        const relative = provider.resolveRelativePath(docPath, imgPath);
        assert.strictEqual(relative, '../img.png');
    });
});
