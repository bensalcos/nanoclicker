"""Read ordinary PE32+ DLL imports without loading or executing the image."""
import re
import struct


def verify_portable_runtime(image: bytes) -> list[str]:
    def unpack(fmt, offset):
        if offset < 0 or offset + struct.calcsize(fmt) > len(image):
            raise ValueError("Truncated PE import metadata.")
        return struct.unpack_from(fmt, image, offset)

    pe, = unpack("<I", 60)
    signature, machine, sections = unpack("<4sHH", pe)
    optional_size, = unpack("<H", pe + 20)
    optional = pe + 24
    if signature != b"PE\0\0" or machine != 0x8664 or optional_size < 128:
        raise ValueError("Expected complete x86_64 PE metadata.")
    magic, = unpack("<H", optional)
    directories, = unpack("<I", optional + 108)
    if magic != 0x20b or directories < 2:
        raise ValueError("Expected PE32+ import directory.")
    table = optional + optional_size

    def offset(rva, size):
        for index in range(sections):
            _, address, raw_size, raw = unpack("<IIII", table + index * 40 + 8)
            relative = rva - address
            if 0 <= relative and relative + size <= raw_size:
                result = raw + relative
                if result + size <= len(image):
                    return result
        raise ValueError("Invalid PE import RVA.")

    rva, size = unpack("<II", optional + 120)
    if not rva and not size:
        return []
    imports = []
    for relative in range(0, size - 19, 20):
        descriptor = unpack("<IIIII", offset(rva + relative, 20))
        if not any(descriptor):
            break
        name = bytearray()
        for index in range(256):
            byte = image[offset(descriptor[3] + index, 1)]
            if not byte:
                break
            name.append(byte)
        else:
            raise ValueError("Unterminated PE import name.")
        try:
            imports.append(name.decode("ascii"))
        except UnicodeDecodeError as error:
            raise ValueError("Invalid PE import name.") from error
    else:
        raise ValueError("Unterminated PE import directory.")
    runtime = [name for name in imports
               if re.match(r"(?i)^(vcruntime|msvcp|concrt|vcomp)\d.*\.dll$", name)]
    if runtime:
        raise ValueError("Portable preview requires external VC++ runtime: " + ", ".join(runtime))
    return imports
