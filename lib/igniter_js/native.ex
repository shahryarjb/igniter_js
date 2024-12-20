defmodule IgniterJs.Native do
  @moduledoc false
  use Rustler, otp_app: :igniter_js, crate: "igniter_js"

  # When your NIF is loaded, it will override this function.
  def is_module_imported_from_ast_nif(_file_content, _module_name), do: error()

  def insert_import_to_ast_nif(_file_content, _import_lines), do: error()

  def remove_import_from_ast_nif(_file_content, _modules), do: error()

  def find_live_socket_node_from_ast_nif(_file_content), do: error()

  def extend_hook_object_to_ast_nif(_file_content, _names), do: error()

  def remove_objects_of_hooks_from_ast_nif(_file_content, _object_names), do: error()

  defp error(), do: :erlang.nif_error(:nif_not_loaded)
end
