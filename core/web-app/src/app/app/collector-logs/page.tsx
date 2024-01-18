'use client';

import React, { useEffect, useState } from 'react';
import PageHeader from '../common/page-header';
import InflowService from '@/services/inflow';
import WASimpleTable from '../common/wa-simple-table';
import { InflowLogItem } from '@/types/inflow-log';
import { createColumnHelper } from '@tanstack/react-table';
import { renderKeyValuePairsWithJson } from '../common/keyvalue-renderer';
import dayjs from 'dayjs';
import { parseDateToDayjs } from '@/utils/date';

const COUNT_OPTIONS = [10, 20, 50, 100];

// Table columns
const columnHelper = createColumnHelper<InflowLogItem>();
const columns = [
  columnHelper.accessor('name', {
    header: () => 'Name',
    cell: (cell) => {
      const name = cell.getValue();
      if (!name) {
        return '(empty)';
      }
      return name;
    },
  }),
  columnHelper.accessor('tags', {
    header: () => 'Tags',
    cell: (cell) => {
      const tags = cell.getValue();
      if (!tags) {
        return '(empty)';
      }
      return (
        <span className="flex flex-col items-center break-all text-start">
          {renderKeyValuePairsWithJson(tags, true)}
        </span>
      );
    },
  }),
  columnHelper.accessor('value', {
    header: () => 'Value',
    // WORKAROUND: cell.getValue() is not working
    cell: (cell) => cell.getValue(),
  }),
  columnHelper.accessor('timestamp', {
    header: () => 'Timestamp',
    cell: (cell) => {
      const EMPTY = '(empty)';
      const timestamp = cell.getValue();
      if (!timestamp) {
        return EMPTY;
      }
      try {
        const date = parseDateToDayjs(timestamp);
        return dayjs(date).format('YYYY-MM-DD HH:mm:ss');
      } catch (error) {
        return EMPTY;
      }
    },
  }),
];

export default function CollectorLogsPage() {
  const [metricIds, setMetricIds] = useState<string[]>([]);
  const [selectedMetricId, setSelectedMetricId] = useState<string>('');
  const [count, setCount] = useState<number>(COUNT_OPTIONS[0]);
  const [metricLogs, setMetricLogs] = useState<any[]>([]);

  // UI
  const [isFetching, setIsFetching] = useState(false);

  // Effects
  useEffect(() => {
    fetchMetricIds();
  }, []);

  useEffect(() => {
    fetchMetricLogs();
  }, [selectedMetricId, count]);

  // Handlers
  const fetchMetricIds = async () => {
    setIsFetching(true);

    try {
      const data = await InflowService.getInflowMetricIds();
      setMetricIds(data);

      if (!selectedMetricId) {
        setSelectedMetricId(data[0]);
      }
    } catch (error) {
      console.error(error);
    }
    setIsFetching(false);
  };

  const fetchMetricLogs = async () => {
    if (!selectedMetricId) {
      return;
    }
    setIsFetching(true);
    try {
      const data = await InflowService.getInflowMetricLogs(
        selectedMetricId,
        count
      );
      console.log(data);
      setMetricLogs(data);
    } catch (error) {
      console.error(error);
    }
    setIsFetching(false);
  };

  const handleMetricIdChange = (
    event: React.ChangeEvent<HTMLSelectElement>
  ) => {
    setSelectedMetricId(event.target.value);
  };

  const handleCountChange = (event: React.ChangeEvent<HTMLSelectElement>) => {
    setCount(parseInt(event.target.value));
  };

  const handleRefresh = async () => {
    await fetchMetricIds();
    await fetchMetricLogs();
  };

  return (
    <main className="flex h-full w-full flex-col">
      {/* Page Header */}
      <PageHeader title="Metrics Collector Logs" />
      {/* Contents */}
      <div className="py-6">
        {/* Controls - Select(metric_id), Select(count)*/}
        <div className="flex flex-row space-x-4 px-6">
          <select
            className="select-bordered select select-sm"
            value={selectedMetricId}
            onChange={handleMetricIdChange}
          >
            <option disabled>Select Metric ID</option>
            {metricIds.map((metricId) => (
              <option key={metricId} value={metricId}>
                {metricId}
              </option>
            ))}
          </select>
          <select
            className="select-bordered select select-sm"
            value={count}
            onChange={handleCountChange}
          >
            {COUNT_OPTIONS.map((count) => (
              <option key={count} value={count}>
                Latest {count} Logs
              </option>
            ))}
          </select>
          {/* Refresh Button */}
          <button
            className="btn-primary btn-sm btn w-32"
            onClick={() => handleRefresh()}
            disabled={isFetching}
          >
            {isFetching ? 'Fetching...' : 'Refresh'}
          </button>
        </div>
        {/* Table */}
        <div className="p-6">
          <div className="wa-card">
            <WASimpleTable<InflowLogItem>
              tableOptions={{
                data: metricLogs,
                columns,
              }}
            />
          </div>
        </div>
      </div>
    </main>
  );
}
