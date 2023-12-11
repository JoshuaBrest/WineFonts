import {
    createWriteStream,
    copyFileSync,
    mkdtempSync,
    readFileSync,
    rmSync,
    statSync,
    writeFileSync,
    existsSync,
    mkdirSync
} from 'fs';
import { extname, join, normalize } from 'path';
import { cwd } from 'process';
import { randomUUID, createHash } from 'crypto';
import fetch from 'node-fetch';
import normalizeUrl from 'normalize-url';
import { tmpdir } from 'os';

const main = async () => {
    const generateUUID = () => randomUUID();
    const version = process.env.VERSION || '0.0.0';
    const urlPrefix = process.env.URL_PREFIX || 'https://example.com/';

    // Out dir
    const outDir = join(cwd(), '..', 'dist');

    // Make sure the out dir exists
    if (!existsSync(outDir)) {
        mkdirSync(outDir);
    } else {
        // Clear the out dir
        rmSync(outDir, { recursive: true, force: true });
        mkdirSync(outDir);
    }

    // Load local Fonts.json file
    let fonts = JSON.parse(
        readFileSync(join(cwd(), '..', 'fonts.json'), 'utf8')
    );

    const font = (i) => {
        return {
            id: i.id,
            name: i.name,
            shortName: i.shortName,
            publisher: i.publisher,
            categories: i.categories.sort(),
            installations: i.installations.sort(sortByDownloadID)
        };
    };

    // Computes a file hash
    const computeFileHash = (filePath) => {
        const hash = createHash('sha256');
        hash.update(readFileSync(filePath));
        return hash.digest('hex');
    };

    // Sorts by download ID
    const sortByDownloadID = (a, b) => {
        // First sort by type
        if (a.type < b.type) return -1;
        else if (a.type > b.type) return 1;

        // Then sort by download ID
        if (a.download < b.download) return -1;
        else if (a.download > b.download) return 1;
        else return 0;
    };

    // Sorts by name
    const sortByName = (a, b) => {
        if (a.name < b.name) return -1;
        else if (a.name > b.name) return 1;
        else return 0;
    };

    const downloads = [];

    // Checks if a download to a local file exists and returns the ID else null
    const downloadExistsLocalFile = async (localFile) => {
        for (const download of downloads) {
            if (
                download._type === 'localFile' &&
                download._localFile === normalize(localFile)
            ) {
                return download.id;
            }
        }
        return null;
    };

    // Checks if a download to a URL exists and returns the ID else null
    const downloadExistsURL = async (url) => {
        for (const download of downloads) {
            if (
                download._type === 'url' &&
                download._url === normalizeUrl(url)
            ) {
                return download.id;
            }
        }
        return null;
    };

    // Adds a local file download
    const addLocalFileDownload = async (localFile) => {
        const localFilePath = join(cwd(), '..', localFile);
        const id = generateUUID();

        // Copy the file to the out dir
        const extension = extname(localFile);
        const outFilePath = join(outDir, id + extension);

        // Copy the file
        copyFileSync(localFilePath, outFilePath);

        downloads.push({
            id: id,
            downloadURL: urlPrefix + encodeURIComponent(id + extension),
            fileSize: statSync(localFilePath).size,
            hash: computeFileHash(localFilePath),
            _type: 'localFile',
            _localFile: normalize(localFile)
        });

        return id;
    };

    // Adds a URL download
    const addURLDownload = async (url) => {
        const tempFolder = mkdtempSync(join(tmpdir(), 'winefonts-'));
        const id = generateUUID();

        // Download the file
        const filePath = join(tempFolder, 'download');
        const data = await fetch(url, {
            headers: {
                Accept: 'application/octet-stream',
                'User-Agent': 'NodeFetch'
            }
        }).catch((err) => {
            console.log('Failed to download file from URL: ' + url);
            throw err;
        });

        // Write the file
        const file = createWriteStream(filePath);

        await new Promise((resolve, reject) => {
            data.body.pipe(file);
            data.body.on('error', (err) => {
                reject(err);
            });
            file.on('finish', () => {
                resolve();
            });
        });

        // Add the download
        downloads.push({
            id: id,
            downloadURL: url,
            fileSize: statSync(filePath).size,
            hash: computeFileHash(filePath),
            _type: 'url',
            _url: normalizeUrl(url)
        });

        // Clean up
        rmSync(tempFolder, { recursive: true, force: true });

        // Return the ID
        return id;
    };

    const addFromObject = async (obj) => {
        const localPath = obj['_localPath'];
        const url = obj['_url'];

        if (typeof localPath !== 'undefined') {
            // Check if the download already exists
            const id = await downloadExistsLocalFile(localPath);

            if (id) {
                obj.download = id;
            } else {
                // Add the download
                obj.download = await addLocalFileDownload(localPath);
            }
        } else if (typeof url !== 'undefined') {
            // Check if the download already exists
            const id = await downloadExistsURL(url);
            if (id) {
                obj.download = id;
            } else {
                // Add the download
                obj.download = await addURLDownload(url);
            }
        }

        // Clean up
        if (localPath) delete obj['_localPath'];
        if (url) delete obj['_url'];
    };

    // Convert debug stuff to production stuff

    // Sort the fonts
    for (const font of fonts.fonts) {
        for (const installation of font.installations) {
            // Add the download
            await addFromObject(installation);
            // Sort the files
            switch (installation.type) {
                case 'cabextract':
                    installation.files = installation.files.sort();
            }
        }
        font.installations = font.installations.sort(sortByDownloadID);
    }
    fonts.fonts = await fonts.fonts.sort(sortByName).map(font);

    // Add downloads
    fonts.downloads = downloads
        .map((download) => {
            return {
                id: download.id,
                downloadURL: download.downloadURL,
                fileSize: download.fileSize,
                hash: download.hash
            };
        })
        .sort((a, b) => {
            if (a.id < b.id) return -1;
            else if (a.id > b.id) return 1;
            else return 0;
        });

    fonts = {
        version: version,
        downloads: fonts.downloads,
        fonts: fonts.fonts
    };

    // Write the JSON file
    writeFileSync(join(outDir, 'fonts.json'), JSON.stringify(fonts));
};

main().catch((err) => {
    console.error(err);
    process.exit(1);
});
