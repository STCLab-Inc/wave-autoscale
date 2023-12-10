import React, { useEffect, useState } from 'react';
import {
  useReactTable,
  flexRender,
  getCoreRowModel,
} from '@tanstack/react-table';
import { TableOptions } from '@tanstack/react-table';
import { RowData } from '@tanstack/react-table';
import Pagination from '@/utils/pagination';

// TODO: Use memo with generic type
export default function WATable<TData extends RowData>({
  tableOptions,
  sizePerPageOptions = [10, 20, 50, 100],
  pagination,
  onChangePageSize,
  onChangePage,
}: {
  tableOptions: Omit<TableOptions<TData>, 'getCoreRowModel'>;
  sizePerPageOptions?: number[];
  pagination: Pagination;
  onChangePageSize: (pageSize: number) => void;
  onChangePage: (page: number) => void;
}) {
  const table = useReactTable({
    ...tableOptions,
    getCoreRowModel: getCoreRowModel(),
  });

  // States
  const [tempCurrentPage, setTempCurrentPage] = useState<number>(
    pagination.currentPage
  );

  // Effects
  useEffect(() => {
    setTempCurrentPage(pagination.currentPage);
  }, [pagination.currentPage]);

  return (
    <div className="flex w-full flex-col">
      <div className="flex items-center justify-end px-8 py-4">
        <div className="mx-4 flex h-8 items-center">
          <div className="mx-2 flex items-center">
            <label className="select-group-sm">
              <select
                value={pagination.pageSize}
                onChange={(event) =>
                  onChangePageSize(Number(event.target.value))
                }
                className="focus:outline-noneselect select-sm max-w-[130px] cursor-pointer rounded-md border border-gray-200 px-2"
              >
                {sizePerPageOptions.map((option, index) => (
                  <option key={index} value={option}>
                    {option}
                  </option>
                ))}
              </select>
            </label>
          </div>
          <div className="min-h-8 mx-2 flex min-w-[100px] items-center justify-center rounded-md border border-gray-200 px-2">
            <input
              className="w-[60px] min-w-[60px] text-center text-sm"
              value={tempCurrentPage}
              onChange={(event) =>
                setTempCurrentPage(Number(event?.target.value))
              }
            />
            <span className="px-2 text-center text-sm">/</span>
            <span className="min-w-[60px] px-4 text-center text-sm">
              {pagination.totalPage}
            </span>
          </div>
          <button
            className={
              tempCurrentPage > pagination.totalPage || tempCurrentPage < 1
                ? 'ml-1 flex h-8 cursor-not-allowed items-center justify-center rounded-md border border-gray-400 bg-gray-400 pl-5 pr-5 text-sm text-gray-50'
                : 'ml-1 flex h-8 cursor-pointer items-center justify-center rounded-md border border-blue-400 bg-blue-400 pl-5 pr-5 text-sm text-gray-50'
            }
            onClick={() => onChangePage(tempCurrentPage)}
            disabled={
              tempCurrentPage > pagination.totalPage || tempCurrentPage < 1
            }
          >
            MOVE
          </button>
        </div>

        <div className="ml-2 flex h-8 items-center">
          <button
            className={
              pagination.currentPage &&
              pagination.totalPage &&
              pagination.currentPage === 1
                ? 'mr-1 flex h-8 cursor-not-allowed items-center justify-center rounded-md border border-gray-400 bg-gray-400 pl-5 pr-5 text-sm text-gray-50'
                : 'mr-1 flex h-8 cursor-pointer items-center justify-center rounded-md border border-red-400 bg-red-400 pl-5 pr-5 text-sm text-gray-50'
            }
            onClick={() => {
              onChangePage(pagination.currentPage - 1);
            }}
            disabled={pagination.currentPage === 1}
          >
            PREVIOUS
          </button>
          <button
            className={
              pagination.currentPage &&
              pagination.totalPage &&
              pagination.currentPage === pagination.totalPage
                ? 'ml-1 flex h-8 cursor-not-allowed items-center justify-center rounded-md border border-gray-400 bg-gray-400 pl-5 pr-5 text-sm text-gray-50'
                : 'ml-1 flex h-8 cursor-pointer items-center justify-center rounded-md border border-blue-400 bg-blue-400 pl-5 pr-5 text-sm text-gray-50'
            }
            onClick={() => {
              onChangePage(pagination.currentPage + 1);
            }}
            disabled={pagination.currentPage === pagination.totalPage}
          >
            NEXT
          </button>
        </div>
      </div>

      <table className="flex w-full flex-col">
        <thead className="text-md flex h-12 w-full items-center justify-between border-b border-t bg-gray-75 py-0 font-bold text-gray-800">
          {table.getHeaderGroups().map((headerGroup) => (
            <tr key={headerGroup.id} className="flex h-full w-full px-8">
              {headerGroup.headers.map((header) => (
                <th
                  key={header.id}
                  className={`flex h-full w-full flex-1 items-center px-4`}
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
        <tbody className="text-md min-h-12 flex w-full flex-col items-center justify-between py-0 text-gray-800">
          {table.getRowModel().rows.map((row) => (
            <tr key={row.id} className="flex w-full border-b px-8 py-4">
              {row.getVisibleCells().map((cell) => (
                <td
                  className={`flex h-full w-full flex-1 items-start px-4`}
                  key={cell.id}
                >
                  {flexRender(cell.column.columnDef.cell, cell.getContext())}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
