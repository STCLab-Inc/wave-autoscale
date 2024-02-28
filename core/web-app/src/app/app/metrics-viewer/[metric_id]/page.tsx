'use client';

import React, { useEffect, useState } from 'react';
import PageHeader from '../../common/page-header';
import MetricsDataService from '@/services/metrics-data';
import WASimpleTable from '../../common/wa-simple-table';
import { createColumnHelper } from '@tanstack/react-table';
import dayjs from 'dayjs';
import { parseDateToDayjs } from '@/utils/date';
import { MetricsDataStats } from '@/types/metrics-data-stats';
import { ResponsiveLine, ResponsiveLineCanvas } from '@nivo/line';

const COUNT_OPTIONS = [10, 20, 50, 100];
const MINUTES_AGO = dayjs().subtract(5, 'minute');
const NUMBER_OF_SECONDS = dayjs().diff(MINUTES_AGO, 'second');
// Table columns
const columnHelper = createColumnHelper<MetricsDataStats>();
const columns = [
  columnHelper.accessor('metricId', {
    header: () => 'Metric ID',
    cell: (cell) => {
      const name = cell.getValue();
      return <span className="font-bold">{name}</span>;
    },
  }),
  columnHelper.accessor('timestamps', {
    header: () => 'Frequency',
    cell: (cell) => {
      const timestamps = cell.getValue();
      const data: { x: number; y: number }[] = [];
      const dataMap: { [key: string]: number } = {};
      timestamps.forEach((timestamp, index) => {
        const dayjsTimestamp = parseDateToDayjs(timestamp);
        if (dayjsTimestamp.isBefore(MINUTES_AGO)) {
          return;
        }
        const key = dayjsTimestamp.diff(MINUTES_AGO, 'second').toString();
        dataMap[key] = 1;
      });
      for (let i = 0; i < NUMBER_OF_SECONDS; i++) {
        const value = dataMap[i.toString()];
        data.push({
          x: i,
          y: value ?? 0,
        });
      }
      return (
        <div className="h-10 w-20">
          <ResponsiveLineCanvas
            data={[
              {
                id: 'timestamps',
                data,
              },
            ]}
            enableArea
            enableGridX={false}
            enableGridY={false}
            enablePoints={false}
            isInteractive={false}
            curve="natural"
            axisBottom={null}
            axisLeft={null}
            xScale={{
              type: 'linear',
              min: 0,
              max: 300,
            }}
          />
        </div>
      );
    },
  }),
  columnHelper.accessor('timestamps', {
    header: () => 'Last Timestamp',
    cell: (cell) => {
      const timestamps = cell.getValue();
      const lastTimestamp = timestamps[timestamps.length - 1];
      return parseDateToDayjs(lastTimestamp).format('YYYY-MM-DD HH:mm:ss');
    },
  }),
  columnHelper.accessor('numberOfValues', {
    header: () => 'The Number of Values',
    cell: (cell) => cell.getValue(),
  }),
  columnHelper.accessor('lastValues', {
    header: () => 'Last Values',
    cell: (cell) => <div className="line-clamp-1">{cell.getValue()}</div>,
  }),
];

export default function MetricsViewerPage() {
  const [data, setData] = useState<MetricsDataStats[]>([]);
  // UI
  const [isFetching, setIsFetching] = useState(false);

  // Effects
  useEffect(() => {
    fetch();
  }, []);

  // Handlers
  const fetch = async () => {
    setIsFetching(true);
    try {
      const data = await MetricsDataService.getStats();
      setData(data);
    } catch (error) {
      console.error(error);
    } finally {
      setIsFetching(false);
    }
  };
  const handleRefresh = async () => {};

  return (
    <main className="flex h-full w-full flex-col">
      {/* Page Header */}
      <PageHeader title="Metrics Viewer" />
      <div className="py-6">
        {/* Table */}
        <div className="p-6">
          <div className="wa-card">
            <WASimpleTable<MetricsDataStats>
              tableOptions={{
                data,
                columns,
              }}
              onRowClick={(row) => {
                alert('row clicked');
              }}
            />
          </div>
        </div>
      </div>
    </main>
  );
}
