# rust-cli-tool-template

This repo serves as a template for CLI tools i create. Some boilerplate is set up in order to get arguments passed by the user.

The `source` and `destination` arguments are for passing input/output through a file instead of `stdin` or `stdout`. These are setup to support relative paths by default for ease of use.

## Using the template

### Git CLI

Using the Git CLI will create the repo locally before making a publically accessable repo on, for example, GitHub.

Run these commands to create a new repo with this tempalte:

```sh
git clone https://github.com/Skyppex/rust-cli-tool-template.git reponame
cd reponame
rm -rf .git
git init
```

This will clone the repo into a folder with the reponame you choose,
cd into it,
then reset git by recreating the `.git` folder.

### GitHub CLI

Using the GitHub CLI you will create the new repo on GitHub which then has to be cloned using git.

Run these commands to create a new repo with this template:

```sh
gh repo create reponame --public --template Skyppex/rust-cli-tool-template --clone
cd reponame
```

Run `gh repo create -h` to see some additional options to pass. Some useful ones are: `--add-readme`, `--description(-d): string`, `--gitignore(-g): string`. There are many more options to pick from :D

### Make changes

Change this readme file to describe your tool rather than the template.

Get coding!