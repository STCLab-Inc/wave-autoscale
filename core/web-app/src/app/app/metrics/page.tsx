'use client';

import React, { useEffect, useState } from 'react';
import { useSearchParams, useRouter } from 'next/navigation';

import MetricService from '@/services/metric';
import { MetricDefinition } from '@/types/bindings/metric-definition';

import MetricDetailDrawer from './metric-drawer';
import ContentHeader from '../common/content-header';
import { TableComponent } from './metric-table';

async function getMetrics() {
  const metrics = await MetricService.getMetrics();
  return metrics;
}

interface MetricDefinitionEx extends MetricDefinition {
  isChecked: boolean;
}

export default function MetricsPage() {
  const router = useRouter();
  const searchParams = useSearchParams();

  const pageParam = searchParams.get('page');
  const page = pageParam ? parseInt(pageParam, 10) : 1;
  const sizeParam = searchParams.get('size');
  const size = sizeParam ? parseInt(sizeParam, 10) : 10;

  const [metrics, setMetrics] = useState<MetricDefinitionEx[]>([]);
  const [metricsItem, setMetricsItem] = useState<MetricDefinitionEx>();

  const fetchMetrics = async () => {
    try {
      const id = metricsItem?.db_id;
      let metricsData = await getMetrics();
      setMetrics(metricsData);
      setMetricsItem(
        metricsData.find((item: MetricDefinitionEx) => item.db_id === id)
      );
    } catch (error) {
      console.error({ error });
      return [];
    }
  };

  useEffect(() => {
    fetchMetrics();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const [selectAll, setSelectAll] = useState(false);

  const handleSelectAll = (e: React.ChangeEvent<HTMLInputElement>) => {
    const checked = e.target.checked;
    setSelectAll(checked);
    const updatedMetricsData = metrics.map((updatedMetricsDataItem) => ({
      ...updatedMetricsDataItem,
      isChecked: checked,
    }));
    setMetrics(updatedMetricsData);
  };

  const SIZE_PER_PAGE_OPTIONS = [10, 50, 100, 200, 500];

  const [sizePerPage, setSizePerPage] = useState(SIZE_PER_PAGE_OPTIONS[0]);
  const [currentPage, setCurrentPage] = useState(1);
  const [totalPage, setTotalPage] = useState(1);

  useEffect(() => {
    setTotalPage(Math.ceil(metrics.length / sizePerPage) || 1);
  }, [metrics, currentPage, totalPage, sizePerPage]);

  const handleSizePerPage = (event: React.ChangeEvent<HTMLSelectElement>) => {
    const newItemsPerPage = parseInt(event.target.value, 10);
    setSizePerPage(newItemsPerPage);
    setCurrentPage(1);
  };

  const handleCurrentPage = (newPage: number) => {
    setCurrentPage(newPage);
  };

  useEffect(() => {
    setCurrentPage(page || 1);
    setSizePerPage(size || SIZE_PER_PAGE_OPTIONS[0]);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [page, size]);

  useEffect(() => {
    if (currentPage > totalPage || currentPage < 1) {
      setCurrentPage(1);
    }
    router.push(`/app/metrics?page=${currentPage}&size=${sizePerPage}`);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [metrics, currentPage, totalPage, sizePerPage, searchParams]);

  const [detailsModalFlag, setDetailsModalFlag] = useState(false);

  const onClickDetails = (metricsItem: MetricDefinitionEx | undefined) => {
    setMetricsItem(metricsItem);
    setDetailsModalFlag(true);
  };

  const [fetchFlag, setFetchFlag] = useState(false);

  useEffect(() => {
    if (fetchFlag) {
      fetchMetrics();
      setFetchFlag(false);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [fetchFlag]);

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
                  onClick={() => onClickDetails(undefined)}
                >
                  ADD METRIC
                </button>
              </div>
            }
          />
          <div className="flex w-full flex-col">
            <TableComponent
              data={metrics}
              setData={setMetrics}
              /*  */
              selectAll={selectAll}
              handleSelectAll={handleSelectAll}
              /*  */
              sizePerPageOptions={SIZE_PER_PAGE_OPTIONS}
              sizePerPage={sizePerPage}
              handleSizePerPage={handleSizePerPage}
              /*  */
              currentPage={currentPage}
              totalPage={totalPage}
              handleCurrentPage={handleCurrentPage}
              /*  */
              onClickDetails={onClickDetails}
            />
          </div>
        </div>
      </div>
      {detailsModalFlag ? (
        <MetricDetailDrawer
          metricsItem={metricsItem}
          setDetailsModalFlag={setDetailsModalFlag}
          setFetchFlag={setFetchFlag}
        />
      ) : null}
    </main>
  );
}
