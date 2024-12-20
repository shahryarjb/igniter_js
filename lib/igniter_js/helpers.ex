defmodule IgniterJS.Helpers do
  @doc """
  Normalize the output of the NIFs.
  """
  defmacro normalize_output(output, caller_function) do
    quote do
      {elem(unquote(output), 0), elem(unquote(caller_function), 0), elem(unquote(output), 2)}
    end
  end

  def read_and_validate_file(file_path) do
    with true <- File.exists?(file_path),
         true <- Path.extname(file_path) in [".js", ".ts"],
         {:ok, file_content} <- File.read(file_path) do
      {:ok, file_content}
    else
      {:error, reason} -> {:error, reason}
      _ -> {:error, "Invalid file path or format."}
    end
  end

  def call_nif_fn(file_path, caller_function, processing_fn) do
    case read_and_validate_file(file_path) do
      {:ok, file_content} ->
        processing_fn.(file_content)
        |> normalize_output(caller_function)

      reason ->
        Tuple.insert_at(reason, 1, :none)
        |> normalize_output(caller_function)
    end
  end
end
