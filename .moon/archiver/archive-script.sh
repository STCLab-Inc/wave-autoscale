#!/bin/sh

# This script is used to archive the binaries for a release

# Check that the VERSION and PLATFORM environment variables are set
if [ -z "$VERSION" ] ; then echo "VERSION is not set" ; exit 1 ; fi
if [ -z "$PLATFORM" ] ; then echo "VERSION is not set" ; exit 1 ; fi

# Create the archive directory
cd ../..
mkdir -p archive/wave-autoscale-$VERSION
mkdir -p archive/wave-autoscale-$VERSION-$PLATFORM

# Copy the binaries
cp -r target/$PLATFORM/release/wave-autoscale archive/wave-autoscale-$VERSION-$PLATFORM
cp -r target/$PLATFORM/release/api-server archive/wave-autoscale-$VERSION-$PLATFORM

# Archive the binaries
tar -C archive -czvf ./archive/wave-autoscale-$VERSION/wave-autoscale-$VERSION-$PLATFORM.tar.gz wave-autoscale-$VERSION-$PLATFORM