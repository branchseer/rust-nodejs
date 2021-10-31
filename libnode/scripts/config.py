import sys

import os

arch_triple_map = {
	"x64": "x86_64",
	"arm64": "aarch64",
	"x86": "i686"
}
platform_triple_map = {
	"linux": "unknown-linux-gnu",
	"win32": "pc-windows-msvc",
	"darwin": "apple-darwin"
}
supported_triples = [
	"i686-pc-windows-msvc",
	"x86_64-pc-windows-msvc",
	"aarch64-pc-windows-msvc",

	"x86_64-apple-darwin",
	"aarch64-apple-darwin",

	"x86_64-unknown-linux-gnu",
	"aarch64-unknown-linux-gnu",
]

nodeVersion = os.environ['LIBNODE_NODE_VERSION']
configFlags = (os.environ.get('LIBNODE_CONFIG_FLAGS') or '').split()

arch = os.environ.get('LIBNODE_ARCH') or "x64"  # x64, arm64, x86

target_triple = f"{arch_triple_map[arch]}-{platform_triple_map[sys.platform]}"
if target_triple not in supported_triples:
	sys.exit(f"Unsupported target: {target_triple}")

zipBasenameSuffix = os.environ.get('LIBNODE_ZIP_SUFFIX', '')

if os.environ.get('LIBNODE_SMALL_ICU', '') == '1':
	configFlags += ['--with-intl=small-icu']
	zipBasenameSuffix += '-small_icu'
