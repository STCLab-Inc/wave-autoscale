import {
  RowData,
  TableOptions,
  flexRender,
  getCoreRowModel,
  useReactTable,
} from '@tanstack/react-table';
import classNames from 'classnames';

export default function WASimpleTable<TData extends RowData>({
  tableOptions,
  onRowClick,
  isLoading,
}: {
  tableOptions: Omit<TableOptions<TData>, 'getCoreRowModel'>;
  onRowClick?: (row: TData) => void;
  isLoading?: boolean;
}) {
  if (isLoading) {
    tableOptions.data = [{} as TData];
  }
  const table = useReactTable({
    ...tableOptions,
    getCoreRowModel: getCoreRowModel(),
  });

  const isClickable = !isLoading && onRowClick;

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
          <tr
            key={row.id}
            className={classNames({
              'cursor-pointer': isClickable,
              'hover:bg-wa-gray-100': isClickable,
            })}
            onClick={
              isClickable
                ? () => {
                    onRowClick(row.original);
                  }
                : undefined
            }
          >
            {row.getVisibleCells().map((cell) => (
              <td className={'first:pl-8'} key={cell.id}>
                {isLoading ? (
                  <div className="skeleton h-5 w-full"></div>
                ) : (
                  flexRender(cell.column.columnDef.cell, cell.getContext())
                )}
              </td>
            ))}
          </tr>
        ))}
      </tbody>
    </table>
  );
}
