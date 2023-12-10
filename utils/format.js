import { readFileSync, writeFileSync } from 'fs';
import { join } from 'path';
import { cwd } from 'process';
import { randomUUID } from 'crypto';

const main = async () => {
    const generateUUID = () => randomUUID();

    const sortByTypeThenLocalPathOrURL = (a, b) => {
        // Sort by type
        if (a.type < b.type) return -1;
        else if (a.type > b.type) return 1;

        // Sort by local path or URL
        const aDownload =
            typeof a._localPath !== 'undefined' ? 'localpath' : 'url';
        const bDownload =
            typeof b._localPath !== 'undefined' ? 'localpath' : 'url';

        // URLs first
        if (aDownload === 'url' && bDownload === 'localpath') return -1;
        else if (aDownload === 'localpath' && bDownload === 'url') return 1;

        const aSort = aDownload === 'url' ? a._url : a._localPath;
        const bSort = bDownload === 'url' ? b._url : b._localPath;

        if (aSort < bSort) return -1;
        else if (aSort > bSort) return 1;
        else return 0;
    };

    const font = (i) => {
        return {
            id: i.id,
            name: i.name,
            shortName: i.shortName,
            publisher: i.publisher,
            categories: i.categories.sort(),
            installations: i.installations.sort(sortByTypeThenLocalPathOrURL)
        };
    };

    const sortByName = (a, b) => {
        if (a.name < b.name) return -1;
        else if (a.name > b.name) return 1;
        else return 0;
    };

    // Load local Fonts.json file
    const fonts = JSON.parse(
        readFileSync(join(cwd(), '..', 'fonts.json'), 'utf8')
    );

    const uuids = [];

    // Format the JSON file

    // Sort the fonts
    for (const font of fonts.fonts) {
        if (font.id === '<UUID>') {
            font.id = generateUUID();
            uuids.push(font.id);
        }

        // Sort first by installation type
        font.installations = font.installations.sort(
            sortByTypeThenLocalPathOrURL
        );

        for (const source of font.installations) {
            // Sort the files
            switch (source.type) {
                case 'cabextract':
                    source.files = source.files.sort();
            }
        }
    }
    fonts.fonts = fonts.fonts.sort(sortByName).map(font);

    // Sort categories
    for (const groups of fonts.groups) {
        groups.fonts = groups.fonts.sort();
    }
    fonts.groups = fonts.groups.sort(sortByName);

    // Write the JSON file
    writeFileSync(
        join(cwd(), '..', 'fonts.json'),
        JSON.stringify(fonts, null, 4)
    );
    uuids.length > 1 &&
        writeFileSync(
            join(cwd(), '..', 'newUUIDs.json'),
            JSON.stringify(uuids, null, 4)
        );
};

main().catch((err) => {
    console.error(err);
    process.exit(1);
});
