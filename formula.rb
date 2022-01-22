class Tmexclude < Formula
  desc "Exclude undesired files (node_modules, target, etc) from your TimeMachine backup"
  homepage "https://github.com/PhotonQuantum/tmexclude"
  url "https://github.com/PhotonQuantum/tmexclude/archive/refs/tags/tmexclude-VERSION.tar.gz"
  sha256 "SHA256"
  license "MIT"

  def install
    bin.install "tmexclude"
    bash_completion.install "completion/tmexclude.bash"
    zsh_completion.install "completion/tmexclude.fish"
    fish_completion.install "completion/_tmexclude"
    inreplace "launch.plist" do |s|
      s.gsub! "LABEL", "#{plist_name}"
      s.gsub! "SELF_PATH", "#{bin}/tmexclude"
    end
    prefix.install_symlink "launch.plist" => "#{plist_name}.plist"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/tmexclude --version")
  end
end