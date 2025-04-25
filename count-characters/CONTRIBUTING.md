# Contributing Guide

- Contributing to The Documentation Compendium is fairly easy. This document shows you how to get started

## General

- Please ensure that any changes you make are in accordance with the [Coding Guidelines](./CODING_GUIDELINES.md) of this repo
- Ensure that your changes are well-documented and tested.
- If you are adding a new tool, please ensure that it is not already in the repo. You can do this by searching the repo for the tool name.

## Submitting changes

- Fork the repo
  - `git clone url-to-your-fork`
- Check out a new branch based and name it to what you intend to do:
  - Example:

    ````sh
    git checkout -b BRANCH_NAME
    ````

    If you get an error, you may need to fetch fooBar first by using

    ````sh
    git remote update && git fetch
    ````

  - Use one branch per fix / feature
- Commit your changes
  - Please provide a git message that explains what you've done
  - Please make sure your commits follow the [conventions](https://gist.github.com/robertpainsi/b632364184e70900af4ab688decf6f53#file-commit-message-guidelines-md)
  - Commit to the forked repository
  - Example:

    ````sh
    git commit -am 'Add some fooBar'
    ````

- Push to the branch
  - Example:

    ````sh
    git push origin BRANCH_NAME
    ````

- Make a pull request
  - Make sure you send the PR to the `fooBar` branch
  - Travis CI is watching you!

If you follow these instructions, your PR will land pretty safely in the main repo!
