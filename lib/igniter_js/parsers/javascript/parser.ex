defmodule IgniterJS.Parsers.Javascript.Parser do
  alias IgniterJS.Native
  require IgniterJS.Helpers
  import IgniterJS.Helpers, only: [call_nif_fn: 4]

  @doc """
  Check if a module is imported in the given file.
  """
  def module_imported?(file_path_or_content, module, type \\ :content) do
    elem(module_imported(file_path_or_content, module, type), 0) == :ok
  end

  def module_imported(file_path_or_content, module, type \\ :content) do
    call_nif_fn(
      file_path_or_content,
      __ENV__.function,
      fn file_content ->
        Native.is_module_imported_from_ast_nif(file_content, module)
      end,
      type
    )
  end

  @doc """
  Insert imports to the given file.
  """
  def insert_imports(file_path_or_content, imports_lines, type \\ :content) do
    call_nif_fn(
      file_path_or_content,
      __ENV__.function,
      fn file_content ->
        Native.insert_import_to_ast_nif(file_content, imports_lines)
      end,
      type
    )
  end

  @doc """
  Remove imports from the given file.
  """
  def remove_imports(file_path_or_content, module, type \\ :content)

  def remove_imports(file_path_or_content, module, type) when is_binary(module) do
    remove_imports(file_path_or_content, [module], type)
  end

  def remove_imports(file_path_or_content, modules, type) when is_list(modules) do
    call_nif_fn(
      file_path_or_content,
      __ENV__.function,
      fn file_content ->
        Native.remove_import_from_ast_nif(file_content, modules)
      end,
      type
    )
  end

  @doc """
  Check if a LiveSocket var exists in the given file.
  """
  def exist_live_socket(file_path_or_content, type \\ :content) do
    call_nif_fn(
      file_path_or_content,
      __ENV__.function,
      fn file_content ->
        Native.find_live_socket_node_from_ast_nif(file_content)
      end,
      type
    )
  end

  def exist_live_socket?(file_path_or_content, type \\ :content) do
    elem(exist_live_socket(file_path_or_content, type), 0) == :ok
  end

  @doc """
  Extend the hook object in the given file.
  """
  def extend_hook_object(file_path_or_content, object_name, type \\ :content)

  def extend_hook_object(file_path_or_content, object_name, type) when is_binary(object_name) do
    extend_hook_object(file_path_or_content, [object_name], type)
  end

  def extend_hook_object(file_path_or_content, objects_names, type) when is_list(objects_names) do
    call_nif_fn(
      file_path_or_content,
      __ENV__.function,
      fn file_content ->
        Native.extend_hook_object_to_ast_nif(file_content, objects_names)
      end,
      type
    )
  end

  @doc """
  Remove objects from the hooks in the given file.
  """
  def remove_objects_from_hooks(file_path_or_content, object_name, type \\ :content)

  def remove_objects_from_hooks(file_path_or_content, object_name, type)
      when is_binary(object_name) do
    remove_objects_from_hooks(file_path_or_content, [object_name], type)
  end

  def remove_objects_from_hooks(file_path_or_content, objects_names, type)
      when is_list(objects_names) do
    call_nif_fn(
      file_path_or_content,
      __ENV__.function,
      fn file_content ->
        Native.remove_objects_of_hooks_from_ast_nif(file_content, objects_names)
      end,
      type
    )
  end
end
