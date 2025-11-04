# How to contribute efficiently

Thank you for taking the time and energy to read this file and contribute to one of the wooga libraries. All contributions are welcome. Be it bug reports, documentation updates, bug fixes or improvements. 

Sections covered in this file:

* [Reporting bugs or proposing features](#reporting-bugs-or-proposing-features)
* [Contributing pull requests](#contributing-pull-requests)
* [Communicating with other developers](#communicating-with--other-developers)

**Please read the first section before reporting a bug!**

## Reporting bugs or proposing features

The golden rule is to **always open *one* issue for *one* bug**. If you notice several bugs and want to report them, make sure to create one new issue for each of them.

Everything referred to hereafter as "bug" also applies for feature requests.

If you are reporting a new issue, you will make our life much simpler (and the fix come much sooner) by following those guidelines:

#### Specify the platform 

If you believe your issue is device/platform dependent, please specify:

* Operating system
* Device (including architecture, e.g. x86, x86_64, arm, etc.)
* Unity version

#### Specify steps to reproduce

Many bugs can't be reproduced unless specific steps are taken. Please **specify the exact steps** that must be taken to reproduce the condition, and try to keep them as minimal as possible.

## Contributing pull requests

If you want to add new engine functionalities, please make sure that:

* This functionality is desired.
* You talked to other developers on how to implement it best (on either communication channel, and maybe in a GitHub issue first before making your PR).
* Even if it does not get merged, your PR is useful for future work by another developer.

Similar rules can be applied when contributing bug fixes - it's always best to discuss the implementation in the bug report first if you are not 100% about what would be the best fix.

#### Be nice to the git history

Try to make simple PRs with that handle one specific topic. Just like for reporting issues, it's better to open 3 different PRs that each address a different issue than one big PR with three commits.

When updating your topic branch with changes from master, please use `git pull --rebase` to avoid creating "merge commits". Those commits unnecessarily pollute the git history when coming from PRs.

```bash
git pull origin master --rebase
```

This will `fetch` the upstream changes, rewind your local checkout to the last integration point, apply the changes from remote and after that apply all your local changes.

You can also set `branch.autosetuprebase` in `git-config`

_from the git man page_
```man
-r, --rebase[=false|true|preserve]
    When true, rebase the current branch on top of the upstream branch
    after fetching. If there is a remote-tracking branch corresponding
    to the upstream branch and the upstream branch was rebased since
    last fetched, the rebase uses that information to avoid rebasing
    non-local changes.

    When preserve, also rebase the current branch on top of the
    upstream branch, but pass --preserve-merges along to git rebase so
    that locally created merge commits will not be flattened.

    When false, merge the current branch into the upstream branch.

    See pull.rebase, branch.<name>.rebase and branch.autosetuprebase in
    git-config(1) if you want to make git pull always use --rebase
    instead of merging.

    Note
    This is a potentially dangerous mode of operation. It rewrites
    history, which does not bode well when you published that
    history already. Do not use this option unless you have read
    git-rebase(1) carefully.
```

**Pro:**
- The history looks cleaner
- no pull merge commits `Merge origin/master into master`

**Cons:**
- you could end up fxing merge issues multiple times (in most cases it's the fault of the first commit not correctly merged) 
- by default the commit dates are no longer in order (there is the flag `--committer-date-is-author-date`)
- you need to stash/unstash local changes (there is a config option `rebase.autoStash` or flag `--autostash` fot that)

This git style guide has some good practices to have in mind: <https://github.com/agis-/git-style-guide>

#### Format your commit logs with readability in mind

The way you format your commit logs is quite important to ensure that the commit history and changelog will be easy to read and understand. A git commit log is formatted as a short title (first line) and an extended description (everything after the first line and an empty separation line).

The short title is the most important part, as it is what will appear in the `shortlog` changelog (one line per commit, so no description shown) or in the GitHub interface unless you click the "expand" button. As the name tells it, try to keep that first line relatively short (ideally <= 50 chars, though it's rare to be able to tell enough in so few characters, so you can go a bit higher) - it should describe what the commit does globally, while details would go in the description. Typically, if you can't keep the title short because you have too much stuff to mention, it means that you should probably split your changes in several commits :)

I could go on and on about the format and length of a good commit message. But check out this post <http://tbaggery.com/2008/04/19/a-note-about-git-commit-messages.html> instead.

In short:

```plain
Capitalized, short (50 chars or less) summary

More detailed explanatory text, if necessary.  Wrap it to about 72 characters
or so.  In some contexts, the first line is treated as the subject of an email
and the rest of the text as the body.  The blank line separating the summary
from the body is critical (unless you omit the body entirely); tools like
rebase can get confused if you run the two together.

Write your commit message in the imperative: "Fix bug" and not "Fixed bug" or 
"Fixes bug."  This convention matches up with commit messages generated by 
commands like git merge and git revert.

Further paragraphs come after blank lines.

- Bullet points are okay, too
- Typically a hyphen or asterisk is used for the bullet, followed by a 
  single space, with blank lines in between, but conventions vary here
- Use a hanging indent

Use markdown when possible. Exceptions are `# Headlines` 
```

Thanks!
