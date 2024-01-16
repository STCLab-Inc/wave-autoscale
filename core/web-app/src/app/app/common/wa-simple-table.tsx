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
    <table className="table-compact table w-full">
      <thead className="bg-wa-gray-200 text-sm font-bold text-wa-gray-900">
        {table.getHeaderGroups().map((headerGroup) => (
          <tr key={headerGroup.id} className="px-6">
            {headerGroup.headers.map((header) => (
              <th
                key={header.id}
                className={'h-12 border-b bg-wa-gray-100 first:pl-8'}
                style={{
                  width: header.getSize(),
                }}
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
      <tbody className="text-sm text-wa-gray-700">
        {table.getRowModel().rows.map((row) => (
          <tr key={row.id} className="">
            {row.getVisibleCells().map((cell) => (
              <td className={'first:pl-8'} key={cell.id}>
                {flexRender(cell.column.columnDef.cell, cell.getContext())}
              </td>
            ))}
          </tr>
        ))}
      </tbody>
    </table>
  );
}
