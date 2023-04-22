import sys, os
import json
import contextlib
import logging

from imagededup.methods import PHash

logging.disable(logging.CRITICAL)

phasher = PHash()

with open(os.devnull, 'w') as f:
    with contextlib.redirect_stderr(f):
        # Generate encodings for all images in an image directory
        encodings = phasher.encode_images(image_dir='./memes/')

        # Find duplicates using the generated encodings
        duplicates = phasher.find_duplicates(encoding_map=encodings)

# Outputs the first duplicate of the provided file
try:
    print(duplicates[sys.argv[1]][0], end='')
except IndexError:
    print(end='')