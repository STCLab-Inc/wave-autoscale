'use client';

import React, { useEffect, useState } from 'react';
import { useSearchParams, useRouter } from 'next/navigation';

import { MetricDefinition } from '@/types/bindings/metric-definition';
import MetricService from '@/services/metric';

import ContentHeader from '../content-header';
import { renderKeyValuePairsWithJson } from '../keyvalue-renderer';
import MetricDetailDrawer from './metric-drawer';

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
  }, []);

  const [checkAllFlag, setCheckAllFlag] = useState(false);

  const handleCheckAllChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const checked = e.target.checked;
    setCheckAllFlag(checked);
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

  const [visibleMetrics, setVisibleMetrics] = useState<MetricDefinitionEx[]>(
    []
  );

  const handleSizePerPageChange = (
    event: React.ChangeEvent<HTMLSelectElement>
  ) => {
    const newItemsPerPage = parseInt(event.target.value, 10);
    setSizePerPage(newItemsPerPage);
    setCurrentPage(1);
  };

  const handlePageChange = (newPage: number) => {
    setCurrentPage(newPage);
  };

  useEffect(() => {
    setCurrentPage(page || 1);
    setSizePerPage(size || SIZE_PER_PAGE_OPTIONS[0]);
  }, [page, size]);

  useEffect(() => {
    if (currentPage > totalPage) {
      setCurrentPage(1);
    }
    router.push(`/app/metrics?page=${currentPage}&size=${sizePerPage}`);
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
  }, [fetchFlag]);

  useEffect(() => {
    setVisibleMetrics(
      metrics.slice(
        (currentPage - 1) * sizePerPage,
        (currentPage - 1) * sizePerPage + sizePerPage
      )
    );
  }, [metrics, currentPage, sizePerPage, totalPage]);

  return (
    <main className="flex h-full w-full flex-row">
      <div className="flex h-full w-full flex-col">
        <div className="flex h-full w-full flex-col">
          <ContentHeader
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
            <div className="flex items-center justify-end px-8 py-4">
              <div className="mx-2 flex h-8 items-center">
                <div className="mr-2 flex items-center">
                  <label className="select-group-sm">
                    <select
                      value={sizePerPage}
                      onChange={handleSizePerPageChange}
                      className="focus:outline-noneselect select-sm max-w-[130px] cursor-pointer rounded-md border border-gray-200 px-2"
                    >
                      {SIZE_PER_PAGE_OPTIONS.map((option, key) => (
                        <option key={key} value={option}>
                          {option}
                        </option>
                      ))}
                    </select>
                  </label>
                </div>
                <div className="min-h-8 mx-2 flex min-w-[100px] items-center justify-center rounded-md border border-gray-200">
                  <span className="px-4 text-center text-sm">
                    {currentPage} / {totalPage}
                  </span>
                </div>
              </div>

              <div className="ml-2 flex h-8 items-center">
                <button
                  className={
                    currentPage === 1
                      ? 'mr-1 flex h-8 cursor-not-allowed items-center justify-center rounded-md border border-gray-400 bg-gray-400 pl-5 pr-5 text-sm text-gray-50'
                      : 'mr-1 flex h-8 cursor-pointer items-center justify-center rounded-md border border-red-400 bg-red-400 pl-5 pr-5 text-sm text-gray-50'
                  }
                  onClick={() => handlePageChange(currentPage - 1)}
                  disabled={currentPage === 1}
                >
                  PREVIOUS
                </button>
                <button
                  className={
                    currentPage && totalPage && currentPage !== totalPage
                      ? 'ml-1 flex h-8 cursor-pointer items-center justify-center rounded-md border border-blue-400 bg-blue-400 pl-5 pr-5 text-sm text-gray-50'
                      : 'ml-1 flex h-8 cursor-not-allowed items-center justify-center rounded-md border border-gray-400 bg-gray-400 pl-5 pr-5 text-sm text-gray-50'
                  }
                  onClick={() => handlePageChange(currentPage + 1)}
                  disabled={currentPage === totalPage}
                >
                  NEXT
                </button>
              </div>
            </div>

            <table className="flex w-full flex-col">
              <thead className="text-md flex h-12 w-full items-center justify-between border-b border-t bg-gray-75 py-0 font-bold text-gray-800">
                <tr className="flex h-full w-full px-8">
                  <th className="mr-4 flex h-full flex-1 items-center">
                    <label className="flex h-full items-center">
                      <input
                        type="checkbox"
                        className="checkbox"
                        checked={checkAllFlag}
                        onChange={handleCheckAllChange}
                      />
                    </label>
                  </th>
                  <th className="mx-4 flex h-full w-full flex-4 items-center">
                    <span className="flex items-center break-words">
                      Metric ID
                    </span>
                  </th>
                  <th className="mx-4 flex h-full w-full flex-2 items-center">
                    <span className="flex items-center break-words">
                      Collector
                    </span>
                  </th>
                  <th className="mx-4 flex h-full w-full flex-10 items-center">
                    <span className="flex items-center break-words">
                      Metadata Values
                    </span>
                  </th>
                  <th className="mx-4 flex h-full w-full flex-1 items-center">
                    <span className="flex items-center break-words">
                      Actions
                    </span>
                  </th>
                </tr>
              </thead>
              <tbody className="text-md min-h-12 flex w-full flex-col items-center justify-between py-0 text-gray-800">
                {visibleMetrics.map((metricsItem: MetricDefinitionEx) => {
                  return (
                    <tr
                      key={metricsItem.db_id}
                      className="flex w-full border-b px-8 py-4"
                    >
                      <td className="mr-4 flex h-full flex-1 items-start">
                        <label className="flex items-center">
                          <input
                            type="checkbox"
                            className="checkbox"
                            checked={metricsItem.isChecked}
                            onChange={(event) => {
                              const checked = event.target.checked;
                              const updatedMetrics = metrics.map((item) =>
                                item.id === metricsItem.id
                                  ? { ...item, isChecked: checked }
                                  : item
                              );
                              setMetrics(updatedMetrics);
                            }}
                          />
                        </label>
                      </td>

                      <td className="mx-4 flex h-full w-full flex-4 items-start">
                        <div className="flex items-center break-all">
                          {metricsItem.id}
                        </div>
                      </td>
                      <td className="mx-4 flex h-full w-full flex-2 items-start">
                        <div className="flex items-center break-all">
                          {metricsItem.collector}
                        </div>
                      </td>
                      <td className="mx-4 flex h-full w-full flex-10 items-start">
                        <div className="flex flex-col items-center">
                          {renderKeyValuePairsWithJson(
                            JSON.stringify(metricsItem.metadata),
                            true
                          )}
                        </div>
                      </td>
                      <td className="mx-4 flex h-full w-full flex-1 items-start">
                        <div className="flex items-center">
                          <button
                            className="badge-info badge bg-[#99E3D0] px-2 py-3 text-white"
                            onClick={() => onClickDetails(metricsItem)}
                          >
                            Details
                          </button>
                        </div>
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
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
