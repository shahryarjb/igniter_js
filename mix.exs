defmodule IgniterJs.MixProject do
  use Mix.Project
  @version "0.0.1"

  def project do
    [
      app: :igniter_js,
      version: @version,
      elixir: "~> 1.17",
      start_permanent: Mix.env() == :prod,
      deps: deps()
    ]
  end

  # Run "mix help compile.app" to learn about applications.
  def application do
    [
      extra_applications: [:logger],
      mod: {IgniterJs.Application, []}
    ]
  end

  # Run "mix help deps" to learn about dependencies.
  defp deps do
    [
      {:rustler, "~> 0.35.1"}
    ]
  end
end
