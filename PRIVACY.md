# Privacy

Last updated: March 4, 2026

## Overview

Lap is designed as a local photo manager. Your photo library is processed on your device, and your files remain under your control.

This document explains what data Lap accesses, what data Lap does not collect by default, and when network access may occur. It is intended as a clear description of how Lap handles data today.

## What Lap Accesses

Lap may access the following data on your device in order to provide its features:

- Photos, videos, and folders that you choose to add to a library
- File metadata such as filenames, paths, timestamps, size, format, EXIF data, ratings, tags, comments, and rotation state
- Generated local app data such as thumbnails, indexes, search data, embeddings, clustering data, and other library-related cache or database records

Lap uses this data to support browsing, search, deduplication, tagging, ratings, face clustering, and other library management features.

## Local Processing

Lap is intended to process your library locally on your device.

By default:

- Your photos and videos are not uploaded to a Lap cloud service
- Lap does not include advertising trackers
- Lap does not send telemetry or analytics about your library usage

## Network Access

Lap may access the network in limited cases where the feature requires it. Based on the current implementation, this may include:

- Checking for application updates
- Downloading application updates from GitHub releases when you choose to install an update
- Opening external links such as the project website or GitHub repository in your browser

If a future feature requires additional network access, it should be documented in the release notes and relevant product documentation.

## Data Storage

Lap stores application data locally on your device. This may include:

- App settings
- Library configuration
- Local database records
- Generated thumbnails and cache data
- Search, deduplication, and related indexing data

This local data is used to provide the app's functionality and improve performance on your device.

## Your Control

You control the libraries and folders that Lap can access.

You can:

- Choose which folders or albums to add
- Remove libraries from the app
- Edit or delete metadata such as ratings, tags, comments, and rotation state
- Delete files using the app's file management features

Removing a library from Lap does not automatically delete your original files unless you explicitly use a delete action.

## Third-Party Services

Lap does not provide its own cloud storage service.

When you use update-related features, release assets may be fetched from GitHub. Those requests are subject to GitHub's terms and privacy practices.

## Changes to This Document

This document may be updated as the app evolves. Privacy-related changes should be reflected in the repository and user-facing documentation.

## Contact

For privacy-related questions or concerns, please open an issue in the project repository:

[https://github.com/julyx10/lap](https://github.com/julyx10/lap)
