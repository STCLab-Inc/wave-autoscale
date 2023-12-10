'use client';

import React, { useEffect, useMemo, useState } from 'react';
import { useSearchParams, useRouter } from 'next/navigation';
import MetricService from '@/services/metric';
import MetricDetailDrawer from './metric-drawer';
import ContentHeader from '../common/content-header';
import { MetricDefinitionEx } from './metric-definition-ex';
import WATable from '../common/wa-table';
import { createColumnHelper } from '@tanstack/react-table';
import { renderKeyValuePairsWithJson } from '../common/keyvalue-renderer';
import Pagination from '@/utils/pagination';
import { decodeTime } from 'ulid';
import dayjs from 'dayjs';

// Constants
const SIZE_PER_PAGE_OPTIONS = [10, 50, 100, 200, 500];

// Utils
async function getMetrics(): Promise<MetricDefinitionEx[]> {
  const metrics = await MetricService.getMetrics();
  return metrics;
}

export default function MetricsPage() {
  const router = useRouter();
  const searchParams = useSearchParams();

  // Query params
  const pageParam = searchParams.get('page');
  const currentPage = pageParam ? parseInt(pageParam, 10) : 1;
  const sizeParam = searchParams.get('size');
  const pageSize = sizeParam
    ? parseInt(sizeParam, 10)
    : SIZE_PER_PAGE_OPTIONS[0];

  // States
  const [metrics, setMetrics] = useState<MetricDefinitionEx[]>([]);
  const [selectedMetric, setSelectedMetric] = useState<MetricDefinitionEx>();
  const [forceFetch, setForceFetch] = useState(false);
  const [detailModalVisible, setDetailModalVisible] = useState(false);
  const [pagination, setPagination] = useState<Pagination>(new Pagination());
  const metricsPerPage = useMemo(() => {
    return metrics.slice(
      (pagination.currentPage - 1) * pagination.pageSize,
      pagination.currentPage * pagination.pageSize
    );
  }, [metrics, pagination.currentPage, pagination.pageSize]);
  // Effects
  useEffect(() => {
    setPagination(new Pagination(pageSize, metrics?.length, currentPage));
  }, [metrics, currentPage, pageSize]);

  useEffect(() => {
    fetchMetrics();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    if (forceFetch) {
      fetchMetrics();
      setForceFetch(false);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [forceFetch]);

  // Handlers
  const fetchMetrics = async () => {
    try {
      const id = selectedMetric?.db_id;
      let metricsData = await getMetrics();
      setMetrics(metricsData);
      setSelectedMetric(
        metricsData.find((item: MetricDefinitionEx) => item.db_id === id)
      );
    } catch (error) {
      console.error({ error });
      return [];
    }
  };

  const columnHelper = createColumnHelper<MetricDefinitionEx>();

  const columns = [
    columnHelper.accessor('id', {
      header: () => 'ID',
      cell: (cell) => cell.getValue(),
    }),
    columnHelper.accessor('collector', {
      header: () => 'Collector',
      cell: (cell) => cell.getValue(),
    }),
    columnHelper.accessor('metadata', {
      header: () => 'Metadata',
      cell: (cell) => (
        <span className="flex flex-col items-center break-all text-start">
          {renderKeyValuePairsWithJson(cell.getValue(), true)}
        </span>
      ),
    }),
    columnHelper.accessor('updated_at', {
      header: () => 'Updated At',
      cell: (cell) => dayjs(cell.getValue()).format('YYYY-MM-DD HH:mm:ss'),
    }),
    columnHelper.display({
      id: 'actions',
      header: () => 'Actions',
      cell: (cell) => (
        <button
          className="flex h-8 items-center justify-center rounded-md border border-blue-400 bg-blue-400  pl-5 pr-5 text-sm text-gray-50"
          onClick={() => {
            setSelectedMetric(cell.row.original);
            setDetailModalVisible(true);
          }}
        >
          Detail
        </button>
      ),
    }),
  ];

  return (
    <main className="flex h-full w-full flex-row">
      <div className="flex h-full w-full flex-col">
        <div className="flex h-full w-full flex-col">
          <ContentHeader
            type="OUTER"
            title="Metrics"
            right={
              <div className="flex items-center">
                <button
                  className="flex h-8 items-center justify-center rounded-md border border-blue-400 bg-blue-400  pl-5 pr-5 text-sm text-gray-50"
                  onClick={() => {
                    setSelectedMetric(undefined);
                    setDetailModalVisible(true);
                  }}
                >
                  ADD METRIC
                </button>
              </div>
            }
          />
          <div className="flex w-full flex-col">
            <WATable<MetricDefinitionEx>
              tableOptions={{
                data: metricsPerPage,
                columns,
              }}
              pagination={pagination}
              onChangePage={(newPage) => {
                router.push(`/app/metrics?page=${newPage}&size=${pageSize}`);
              }}
              onChangePageSize={(newSize) => {
                router.push(`/app/metrics?page=${currentPage}&size=${newSize}`);
              }}
            />
            {/* <TableComponent
              tableFormat={tableFormat}
              data={metrics}
              sizePerPageOptions={SIZE_PER_PAGE_OPTIONS}
              sizePerPage={sizePerPage}
              handleSizePerPage={handleSizePerPage}
              currentPage={currentPage}
              totalPage={totalPage}
              handleCurrentPage={handleCurrentPage}
            /> */}
          </div>
        </div>
      </div>
      {detailModalVisible ? (
        <MetricDetailDrawer
          metric={selectedMetric}
          onClose={() => setDetailModalVisible(false)}
          onChange={() => setForceFetch(true)}
        />
      ) : null}
    </main>
  );
}
