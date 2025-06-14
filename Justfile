list:
	just --list

coverage OPEN='':
  #!/usr/bin/env fish
  set -x RUSTFLAGS '-C instrument-coverage'
  set -x LLVM_PROFILE_FILE (pwd)'/scratch/'(whoami)'-%p-%m.profraw'
  # Using the for loop avoids warnings in output
  for file in (pwd)/scratch/*.profraw; rm $file; end
  cargo test --tests
  grcov . -s . --binary-path ./target/debug/ -t html --branch --ignore-not-existing -o ./target/debug/coverage/ --excl-start '^//\s*\{grcov-excl-start\}' --excl-stop '^//\s*\{grcov-excl-end\}'
  cp ./target/debug/coverage/coverage.json ./coverage.json
  if string match -r 'open$' -- '{{OPEN}}'
    open target/debug/coverage/index.html
  end

doc OPEN='':
  #!/usr/bin/env fish
  if string match -r 'open$' -- '{{OPEN}}'
    cargo doc --open
  else
    cargo doc
  end

cross:
	#!/usr/bin/env fish
	cross build --release --target aarch64-unknown-linux-gnu
	cross build --release --target arm-unknown-linux-gnueabihf

pack:
	#!/usr/bin/env fish
	rm scratch/*.tar.gz
	pushd target/aarch64-unknown-linux-gnu/release
	tar --no-xattrs -cvzf ../../../scratch/prusa_connect_uploader-0.1.0-aarch64-unknown-linux-gnu.tar.gz prusa-connect-uploader
	popd
	pushd target/arm-unknown-linux-gnueabihf/release/
	tar --no-xattrs -cvzf ../../../scratch/prusa_connect_uploader-0.1.0-arm-unknown-linux-gnueabihf.tar.gz prusa-connect-uploader
	popd

upload FLAVOR='' USER_AT_HOST='':
	#!/usr/bin/env fish
	set flavor {{FLAVOR}}
	set user_at_host {{USER_AT_HOST}}
	if test -z "$flavor" -o -z "$user_at_host"
		echo Supply FLAVOR and USER_AT_HOST
		exit 1
	end

	switch $flavor
	case aarch64
		scp (pwd)/target/aarch64-unknown-linux-gnu/release/prusa-connect-uploader "$user_at_host":
	case arm
		scp (pwd)/target/arm-unknown-linux-gnueabihf/release/prusa-connect-uploader "$user_at_host":
	case '*'
		echo Choose aarch64 or arm architectures
		exit 1
	end

release OPERATION='incrPatch':
  #!/usr/bin/env fish
  function info
    set_color green; echo "👉 "$argv; set_color normal
  end
  function warning
    set_color yellow; echo "🐓 "$argv; set_color normal
  end
  function error
    set_color red; echo "💥 "$argv; set_color normal
  end

  if test ! -e "Cargo.toml"
    error "Cargo.toml file not found"
    exit 1
  end

  info "Checking for uncommitted changes"

  if not git diff-index --quiet HEAD -- > /dev/null 2> /dev/null
    error "There are uncomitted changes - commit or stash them and try again"
    exit 1
  end

  set branch (string trim (git rev-parse --abbrev-ref HEAD 2> /dev/null))
  set name (basename (pwd))

  info "Starting release of '"$name"' on branch '"$branch"'"

  info "Checking out '"$branch"'"
  git checkout $branch

  info "Pulling latest"
  git pull

  mkdir scratch 2> /dev/null

  if not stampver {{OPERATION}} -u -i version.json5
    error "Unable to generation version information"
    exit 1
  end

  set tagName (cat "scratch/version.tag.txt")
  set tagDescription (cat "scratch/version.desc.txt")

  git rev-parse $tagName > /dev/null 2> /dev/null
  if test $status -ne 0; set isNewTag 1; end

  if set -q isNewTag
    info "'"$tagName"' is a new tag"
  else
    warning "Tag '"$tagName"' already exists and will not be moved"
  end

  if test -e 'justfile' -o -e 'Justfile'
    just coverage
  else
    cargo test
  end

  if test $status -ne 0
    # Rollback
    git checkout $branch .
    error "Tests failed '"$name"' on branch '"$branch"'"
    exit 1
  end

  info "Staging version changes"
  git add :/

  info "Committing version changes"
  git commit -m $tagDescription

  if set -q isNewTag
    info "Tagging"
    git tag -a $tagName -m $tagDescription
  end

  info "Pushing to 'origin'"
  git push --follow-tags

  info "Finished release of '"$name"' on branch '"$branch"'. You can publish the crate."
  exit 0
