'use client';

import React, { useState } from 'react';
import PageHeader from '../../common/page-header';
import { useMetricsData } from '@/services/metrics-data';
import WAVirtualizedTable from '../../common/wa-virtualized-table';
import { createColumnHelper } from '@tanstack/react-table';
import { parseDateToDayjs } from '@/utils/date';
import { MetricsDataItem } from '@/types/metrics-data-stats';
import WATinyAreaChart from '../../common/wa-tiny-area-chart';
import { debounce } from 'lodash';
import WAAreaChart from '../../common/wa-area-chart';
import { successToast } from '@/utils/toast';
import { generateGetCode } from '@/utils/code';

let metricIdForCopy = '';

const DETAIL_MODAL_ID = 'metrics-viewer-detail-modal';

// Table columns
const columnHelper = createColumnHelper<MetricsDataItem>();
const columns = [
  columnHelper.accessor('name', {
    header: () => 'Name',
    cell: (cell: any) => {
      const name = cell.getValue();
      return (
        <span className="break-all font-bold" style={{ width: '16rem' }}>
          {name}
        </span>
      );
    },
  }),
  columnHelper.accessor('tags', {
    header: () => 'Tags',
    cell: (cell: any) => {
      const tags = cell.getValue();
      return (
        <div className="break-all" style={{ width: '20rem' }}>
          {JSON.stringify(tags)}
        </div>
      );
    },
  }),
  columnHelper.accessor('values', {
    header: () => 'Trends',
    cell: (cell: any) => {
      const values = cell.getValue();
      return (
        <div className="h-6 w-24">
          <WATinyAreaChart data={values} dataKey="value" />
        </div>
      );
    },
  }),
  columnHelper.accessor('values', {
    header: () => 'Last Timestamp',
    cell: (cell: any) => {
      const values = cell.getValue();
      const lastTimestamp = values[values.length - 1].timestamp;
      return parseDateToDayjs(lastTimestamp).format('YYYY-MM-DD HH:mm:ss');
    },
  }),
  columnHelper.accessor('values', {
    header: () => 'Last Value',
    cell: (cell: any) => {
      const values = cell.getValue();
      const lastValue = values[values.length - 1].value;
      return lastValue;
    },
  }),
  columnHelper.display({
    id: 'code',
    header: () => 'Expression Code',
    size: 50,
    cell: (cell) => {
      return (
        // Copy to clipboard
        <button
          className="btn btn-xs"
          onClick={(event) => {
            event.stopPropagation();
            const original = cell.row.original;
            const code = generateGetCode({
              metricId: metricIdForCopy,
              name: original.name,
              tags: original.tags,
            });
            navigator.clipboard.writeText(code);
            successToast('Code copied to clipboard');
            return false;
          }}
        >
          Copy
        </button>
      );
    },
  }),
];

export default function MetricsViewerDetailPage({
  params,
}: {
  params: { metric_id: string };
}) {
  const metricId = params.metric_id;
  metricIdForCopy = metricId;

  // UI
  const [search, setSearch] = useState<RegExp>();
  const [selectedRow, setSelectedRow] = useState<MetricsDataItem | null>(null);

  // Data
  const { data, isLoading } = useMetricsData(metricId);
  const filteredData = data?.filter((item) => {
    if (!search) return true;
    if (item.name && search.test(item.name)) return true;
    if (item.tags && search.test(JSON.stringify(item.tags))) return true;
    return false;
  });

  // Handlers
  const handleChangeSearch = debounce(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const search = e.target.value;
      setSearch(new RegExp(search, 'i'));
    },
    500
  );

  return (
    <main className="flex h-full w-full flex-col">
      {/* Page Header */}
      <PageHeader title="Metrics Viewer" backButton subtitle={metricId} />
      <div className="flex flex-1 flex-col py-6">
        {/* Controls */}
        <div className="flex px-6">
          {/* Search */}
          <div className="w-[300px]">
            <label className="input-bordered input input-sm flex items-center gap-2">
              <input
                type="text"
                className="grow"
                placeholder="Search"
                onChange={handleChangeSearch}
              />
              <svg
                xmlns="http://www.w3.org/2000/svg"
                viewBox="0 0 16 16"
                fill=""
                className="h-4 w-4 opacity-70"
              >
                <path
                  fillRule="evenodd"
                  d="M9.965 11.026a5 5 0 1 1 1.06-1.06l2.755 2.754a.75.75 0 1 1-1.06 1.06l-2.755-2.754ZM10.5 7a3.5 3.5 0 1 1-7 0 3.5 3.5 0 0 1 7 0Z"
                  clipRule="evenodd"
                />
              </svg>
            </label>
          </div>
        </div>
        {/* Table */}
        <div className="flex flex-1 flex-col p-6">
          <div className="wa-card flex-1">
            <WAVirtualizedTable<MetricsDataItem>
              tableOptions={{
                data: filteredData ?? [],
                columns,
              }}
              isLoading={isLoading}
              onRowClick={(row) => {
                setSelectedRow(row);
                (document.getElementById(DETAIL_MODAL_ID) as any).showModal();
              }}
            />
          </div>
        </div>
      </div>
      {/* Detail Modal */}
      <dialog id={DETAIL_MODAL_ID} className="modal">
        <div className="modal-box flex h-3/5 w-9/12 max-w-5xl flex-col overflow-hidden">
          <form method="dialog">
            {/* if there is a button in form, it will close the modal */}
            <button className="btn-ghost btn btn-sm btn-circle absolute right-2 top-2">
              ✕
            </button>
          </form>
          <div className="flex flex-1 flex-col">
            <h3 className="text-lg font-bold">{selectedRow?.name}</h3>
            <div className="w-full flex-1 p-4">
              <WAAreaChart
                data={selectedRow?.values}
                yDataKey="value"
                xDataKey="timestamp"
              />
            </div>
          </div>
        </div>
        <form method="dialog" className="modal-backdrop">
          <button>close</button>
        </form>
      </dialog>
    </main>
  );
}
