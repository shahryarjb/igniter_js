defmodule IgniterJS.MixProject do
  use Mix.Project
  @version "0.0.1"
  @source_url "https://github.com/ash-project/igniter_js"

  def project do
    [
      app: :igniter_js,
      version: @version,
      elixir: "~> 1.17",
      name: "IgniterJS",
      elixirc_paths: elixirc_paths(Mix.env()),
      start_permanent: Mix.env() == :prod,
      deps: deps(),
      description: description(),
      package: package(),
      source_url: @source_url
    ]
  end

  defp description() do
    "Codemods for JavaScript in Elixir, powered by a high-performance Rust parser integrated via NIFs"
  end

  defp package() do
    [
      files: ~w(lib .formatter.exs mix.exs LICENSE README*),
      licenses: ["MIT"],
      links: %{
        "GitHub" => @source_url,
        "Discord" => "https://discord.gg/HTHRaaVPUc",
        "Website" => "https://ash-hq.org",
        "Forum" => "https://elixirforum.com/c/ash-framework-forum/",
        "Changelog" => "#{@source_url}/blob/main/CHANGELOG.md"
      }
    ]
  end

  defp elixirc_paths(:test), do: ["lib", "test/support"]
  defp elixirc_paths(_), do: ["lib"]

  # Run "mix help compile.app" to learn about applications.
  def application do
    [
      extra_applications: [:logger],
      mod: {IgniterJS.Application, []}
    ]
  end

  # Run "mix help deps" to learn about dependencies.
  defp deps do
    [
      {:rustler, "~> 0.35.1"},
      {:ex_doc, "~> 0.35.1", only: [:dev, :test], runtime: false}
    ]
  end
end
