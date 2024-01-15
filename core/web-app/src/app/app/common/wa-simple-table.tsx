import {
  RowData,
  TableOptions,
  flexRender,
  getCoreRowModel,
  useReactTable,
} from '@tanstack/react-table';

export default function WASimpleTable<TData extends RowData>({
  tableOptions,
}: {
  tableOptions: Omit<TableOptions<TData>, 'getCoreRowModel'>;
}) {
  const table = useReactTable({
    ...tableOptions,
    getCoreRowModel: getCoreRowModel(),
  });

  return (
    <table className="flex w-full flex-col">
      <thead className="flex h-12 w-full items-center justify-between border-b border-t bg-wa-gray-200 py-0 text-sm font-bold text-wa-gray-900">
        {table.getHeaderGroups().map((headerGroup) => (
          <tr key={headerGroup.id} className="flex h-full w-full px-6">
            {headerGroup.headers.map((header) => (
              <th
                key={header.id}
                className={`flex h-full w-full flex-1 items-center`}
              >
                {header.isPlaceholder
                  ? null
                  : flexRender(
                      header.column.columnDef.header,
                      header.getContext()
                    )}
              </th>
            ))}
          </tr>
        ))}
      </thead>
      <tbody className="min-h-12 flex w-full flex-col items-center justify-between py-0 text-sm text-wa-gray-700">
        {table.getRowModel().rows.map((row) => (
          <tr key={row.id} className="flex w-full border-b py-2">
            {row.getVisibleCells().map((cell) => (
              <td
                className={`flex h-full w-full flex-1 items-start px-2`}
                key={cell.id}
              >
                {flexRender(cell.column.columnDef.cell, cell.getContext())}
              </td>
            ))}
          </tr>
        ))}
      </tbody>
    </table>
  );
}
