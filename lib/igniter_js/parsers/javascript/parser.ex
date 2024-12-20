defmodule IgniterJS.Parsers.Javascript.Parser do
  alias IgniterJS.Native
  require IgniterJS.Helpers
  import IgniterJS.Helpers, only: [call_nif_fn: 3]

  @doc """
  Check if a module is imported in the given file.
  """
  def module_imported?(file_path, module) when is_binary(module) do
    elem(module_imported(file_path, module), 0) == :ok
  end

  def module_imported(file_path, module) when is_binary(module) do
    call_nif_fn(file_path, __ENV__.function, fn file_content ->
      Native.is_module_imported_from_ast_nif(file_content, module)
    end)
  end

  @doc """
  Insert imports to the given file.
  """
  def insert_imports(file_path, imports_lines) when is_binary(imports_lines) do
    call_nif_fn(file_path, __ENV__.function, fn file_content ->
      Native.insert_import_to_ast_nif(file_content, imports_lines)
    end)
  end

  @doc """
  Remove imports from the given file.
  """
  def remove_imports(file_path, module) when is_binary(module) do
    remove_imports(file_path, [module])
  end

  def remove_imports(file_path, modules) when is_list(modules) do
    call_nif_fn(file_path, __ENV__.function, fn file_content ->
      Native.remove_import_from_ast_nif(file_content, modules)
    end)
  end

  @doc """
  Check if a LiveSocket var exists in the given file.
  """
  def exist_live_socket(file_path) do
    call_nif_fn(file_path, __ENV__.function, fn file_content ->
      Native.find_live_socket_node_from_ast_nif(file_content)
    end)
  end

  def exist_live_socket?(file_path) do
    elem(exist_live_socket(file_path), 0) == :ok
  end

  @doc """
  Extend the hook object in the given file.
  """
  def extend_hook_object(file_path, object_name) when is_binary(object_name) do
    extend_hook_object(file_path, [object_name])
  end

  def extend_hook_object(file_path, objects_names) when is_list(objects_names) do
    call_nif_fn(file_path, __ENV__.function, fn file_content ->
      Native.extend_hook_object_to_ast_nif(file_content, objects_names)
    end)
  end

  @doc """
  Remove objects from the hooks in the given file.
  """
  def remove_objects_from_hooks(file_path, object_name) when is_binary(object_name) do
    remove_objects_from_hooks(file_path, [object_name])
  end

  def remove_objects_from_hooks(file_path, objects_names) when is_list(objects_names) do
    call_nif_fn(file_path, __ENV__.function, fn file_content ->
      Native.remove_objects_of_hooks_from_ast_nif(file_content, objects_names)
    end)
  end
end
