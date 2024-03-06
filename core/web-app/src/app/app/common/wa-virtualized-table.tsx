import {
  Row,
  RowData,
  TableOptions,
  flexRender,
  getCoreRowModel,
  useReactTable,
} from '@tanstack/react-table';
import { useVirtualizer } from '@tanstack/react-virtual';
import classNames from 'classnames';
import _ from 'lodash';
import { memo, useRef } from 'react';

const NUMBER_OF_SKELETONS = 5;
const SKELETON_DATA = _.range(NUMBER_OF_SKELETONS).map(() => ({} as RowData));
/**
WAVirtualizedTable is a virtualized table component that uses tanstack/react-table and tanstack/react-virtual

It should be used with a element that has a fixed width and height
 */
function WAVirtualizedTable<TData extends RowData>({
  tableOptions,
  onRowClick,
  isLoading,
  rowHeight = 40,
}: {
  tableOptions: Omit<TableOptions<TData>, 'getCoreRowModel'>;
  onRowClick?: (row: TData) => void;
  isLoading?: boolean;
  rowHeight?: number;
}) {
  if (isLoading) {
    tableOptions.data = SKELETON_DATA as TData[];
  }
  const table = useReactTable({
    ...tableOptions,
    getCoreRowModel: getCoreRowModel(),
  });

  const { rows } = table.getRowModel();
  const parentRef = useRef<HTMLDivElement>(null);

  const virtualizer = useVirtualizer({
    count: rows.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => rowHeight,
    overscan: 20,
  });

  const isClickable = !isLoading && onRowClick;

  return (
    <div className="relative h-full w-full">
      <div className="absolute h-full w-full overflow-auto" ref={parentRef}>
        <div
          className="relative"
          style={{
            height: `${virtualizer.getTotalSize()}px`,
          }}
        >
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
            <tbody className="bg-white text-sm text-wa-gray-700">
              {virtualizer.getVirtualItems().map((virtualRow, index) => {
                const row = rows[virtualRow.index] as Row<TData>;
                return (
                  <tr
                    key={row.id}
                    //needed for dynamic row height measurement
                    data-index={virtualRow.index}
                    //measure dynamic row height
                    ref={(node) => virtualizer.measureElement(node)}
                    className={classNames({
                      'cursor-pointer': isClickable,
                      hover: isClickable,
                    })}
                    style={{
                      height: `${virtualRow.size}px`,
                      maxHeight: `${virtualRow.size}px`,
                      transform: `translateY(${
                        virtualRow.start - index * virtualRow.size
                      }px)`,
                    }}
                    onClick={
                      isClickable
                        ? () => {
                            onRowClick(row.original);
                          }
                        : undefined
                    }
                  >
                    {row.getVisibleCells().map((cell) => (
                      <td className={classNames('first:pl-8')} key={cell.id}>
                        {isLoading ? (
                          <div className="skeleton h-5 w-full"></div>
                        ) : (
                          flexRender(
                            cell.column.columnDef.cell,
                            cell.getContext()
                          )
                        )}
                      </td>
                    ))}
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}

export default memo(WAVirtualizedTable) as typeof WAVirtualizedTable;
