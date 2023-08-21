#!/bin/sh

# This script is used to archive the binaries for a release

# Check that the VERSION and PLATFORM environment variables are set
if [ -z "$VERSION" ] ; then echo "VERSION is not set" ; exit 1 ; fi
if [ -z "$PLATFORM" ] ; then echo "VERSION is not set" ; exit 1 ; fi

# Create the archive directory
cd ../..
rm -rf archive/wave-autoscale-$VERSION/wave-autoscale-$VERSION-$PLATFORM
mkdir -p archive/wave-autoscale-$VERSION/wave-autoscale-$VERSION-$PLATFORM
# mkdir -p archive/wave-autoscale-$VERSION-$PLATFORM

# Copy the binaries
cp -r target/$PLATFORM/release/wave-controller archive/wave-autoscale-$VERSION/wave-autoscale-$VERSION-$PLATFORM
cp -r target/$PLATFORM/release/wave-api-server archive/wave-autoscale-$VERSION/wave-autoscale-$VERSION-$PLATFORM
cp -r core/web-app/.next/standalone archive/wave-autoscale-$VERSION/wave-autoscale-$VERSION-$PLATFORM/wave-web-app
cp -r core/web-app/.next/static archive/wave-autoscale-$VERSION/wave-autoscale-$VERSION-$PLATFORM/wave-web-app/.next/static

# Copy the configuration files
cp -r core/wave-autoscale/tests/config/wave-config.yaml archive/wave-autoscale-$VERSION/wave-autoscale-$VERSION-$PLATFORM

# Archive the binaries
tar -C archive -czvf ./archive/wave-autoscale-$VERSION/wave-autoscale-$VERSION-$PLATFORM.tar.gz wave-autoscale-$VERSION/wave-autoscale-$VERSION-$PLATFORM