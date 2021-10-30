# libnode 

[![release.yml workflow status](https://github.com/patr0nus/libnode/workflows/Release/badge.svg)](https://github.com/patr0nus/libnode/actions/workflows/release.yml)

This repo contains the scripts that build [Node.js](http://nodejs.org/) as a static library for embedding in [DeskGap](https://deskgap.com/).

## Usage

### Configuring

#### Specify the Node version:
```sh
export LIBNODE_NODE_VERSION=v17.0.1
```

#### Remove `Intl` support to reduce the size (optional):
```sh
export LIBNODE_CONFIG_FLAGS=--without-intl
export LIBNODE_ZIP_SUFFIX=-nointl
```

#### Build the x86 version (optional, Windows only):
```sh
export LIBNODE_ARCH=x86
```

### Downloading the source code of Node.js:
```sh
python3 -m scripts.download
```

### Patch the source code:
```sh
python3 -m scripts.patch
```

### Building Node.js:
```sh
python3 -m scripts.build
```

### Postprocessing the static library files:
```sh
python3 -m scripts.postproc
```

### Testing the library:
```sh
python3 -m scripts.test
```

### Archiving:
```sh
python3 -m scripts.archive
```
