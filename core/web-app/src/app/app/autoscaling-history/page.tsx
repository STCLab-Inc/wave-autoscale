'use client';

import React, { useEffect, useMemo, useState } from 'react';
import { useSearchParams, useRouter } from 'next/navigation';

import dayjs, { Dayjs } from 'dayjs';
import { decodeTime } from 'ulid';

import AutoscalingHistoryService from '@/services/autoscaling-history';
import { AutoscalingHistoryDefinition } from '@/types/bindings/autoscaling-history-definition';

import AutoscalingHistoryHeatmap from './autoscaling-history-heatmap';
import AutoscalingHistoryDetailDrawer from './autoscaling-history-drawer';
import ContentHeader from '../common/content-header';
import { renderKeyValuePairsWithJson } from '../common/keyvalue-renderer';

const formatDate = (date: Dayjs) => date.format('YYYY-MM-DD');

async function getAutoscalingHistory(from: Dayjs, to: Dayjs) {
  const autoscalingHistory =
    await AutoscalingHistoryService.getAutoscalingHistoryByFromTo(from, to);
  return autoscalingHistory;
}

const DEFAULT_FROM = dayjs().subtract(7, 'days');
const DEFAULT_TO = dayjs();

interface AutoscalingHistoryDefinitionEx extends AutoscalingHistoryDefinition {
  created_at: string;
  isChecked: boolean;
}

export default function AutoscalingHistoryPage() {
  const router = useRouter();
  const searchParams = useSearchParams();

  const fromParam = searchParams.get('from');
  const toParam = searchParams.get('to');
  const from = fromParam || formatDate(DEFAULT_FROM);
  const to = toParam || formatDate(DEFAULT_TO);
  const fromDayjs = useMemo(() => dayjs(from).startOf('day'), [from]);
  const toDayjs = useMemo(() => dayjs(to).endOf('day'), [to]);

  const pageParam = searchParams.get('page');
  const page = pageParam ? parseInt(pageParam, 10) : 1;
  const sizeParam = searchParams.get('size');
  const size = sizeParam ? parseInt(sizeParam, 10) : 10;

  const [autoscalingHistory, setAutoscalingHistory] = useState<
    AutoscalingHistoryDefinitionEx[]
  >([]);
  const [autoscalingHistoryItem, setAutoscalingHistoryItem] =
    useState<AutoscalingHistoryDefinitionEx>();

  const fetchAutoscalingHistory = async () => {
    try {
      const id = autoscalingHistoryItem?.id;
      let autoscalingHistoryData = await getAutoscalingHistory(
        fromDayjs,
        toDayjs
      );
      autoscalingHistoryData = autoscalingHistoryData.map(
        (autoscalingHistoryDataItem: AutoscalingHistoryDefinition) => ({
          ...autoscalingHistoryDataItem,
          created_at: decodeTime(autoscalingHistoryDataItem.id),
        })
      );
      setAutoscalingHistory(autoscalingHistoryData);
      setAutoscalingHistoryItem(
        autoscalingHistoryData.find(
          (item: AutoscalingHistoryDefinition) => item.id === id
        )
      );
    } catch (error) {
      console.error({ error });
      return [];
    }
  };

  useEffect(() => {
    fetchAutoscalingHistory();
  }, [fromDayjs, toDayjs]);

  const handleDateChange = (field: 'from' | 'to', value: string) => {
    const newDate = dayjs(value);
    const isValidDate = newDate.isValid();

    if (isValidDate) {
      const params = {
        from: field === 'from' ? formatDate(newDate) : from,
        to: field === 'to' ? formatDate(newDate) : to,
      };
      router.push(
        `/app/autoscaling-history?from=${params.from}&to=${
          params.to
        }&page=${1}&view=${sizePerPage}`
      );
    }
  };

  const [selectAll, setSelectAll] = useState(false);

  const handleSelectAll = (event: React.ChangeEvent<HTMLInputElement>) => {
    const checked = event.target.checked;
    setSelectAll(checked);
    const updatedAutoscalingHistoryData = autoscalingHistory.map(
      (updatedAutoscalingHistoryDataItem) => ({
        ...updatedAutoscalingHistoryDataItem,
        isChecked: checked,
      })
    );
    setAutoscalingHistory(updatedAutoscalingHistoryData);
  };

  const SIZE_PER_PAGE_OPTIONS = [10, 50, 100, 200, 500];

  const [sizePerPage, setSizePerPage] = useState(SIZE_PER_PAGE_OPTIONS[0]);
  const [currentPage, setCurrentPage] = useState(1);
  const [totalPage, setTotalPage] = useState(1);

  useEffect(() => {
    setTotalPage(Math.ceil(autoscalingHistory.length / sizePerPage) || 1);
  }, [autoscalingHistory, from, to, currentPage, totalPage, sizePerPage]);

  const [visibleAutoscalingHistory, setVisibleAutoscalingHistory] = useState<
    AutoscalingHistoryDefinitionEx[]
  >([]);

  const handleSizePerPage = (event: React.ChangeEvent<HTMLSelectElement>) => {
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
    router.push(
      `/app/autoscaling-history?from=${from}&to=${to}&page=${currentPage}&size=${sizePerPage}`
    );
  }, [
    autoscalingHistory,
    from,
    to,
    currentPage,
    totalPage,
    sizePerPage,
    searchParams,
  ]);

  const [detailsModalFlag, setDetailsModalFlag] = useState(false);

  const onClickDetails = (
    autoscalingHistoryItem: AutoscalingHistoryDefinitionEx
  ) => {
    setAutoscalingHistoryItem(autoscalingHistoryItem);
    setDetailsModalFlag(true);
  };

  const [fetchFlag, setFetchFlag] = useState(false);

  useEffect(() => {
    if (fetchFlag) {
      fetchAutoscalingHistory();
      setFetchFlag(false);
    }
  }, [fetchFlag]);

  useEffect(() => {
    setVisibleAutoscalingHistory(
      autoscalingHistory.slice(
        (currentPage - 1) * sizePerPage,
        (currentPage - 1) * sizePerPage + sizePerPage
      )
    );
  }, [autoscalingHistory, currentPage, sizePerPage, totalPage]);

  return (
    <main className="flex h-full w-full flex-row">
      <div className="flex h-full w-full flex-col">
        <div className="flex h-full w-full flex-col">
          <ContentHeader
            type="OUTER"
            title="Autoscaling History"
            right={
              <div className="flex items-center">
                <div className="form-control mr-2">
                  <label className="input-group-sm">
                    <input
                      type="date"
                      className="input-bordered input input-sm max-w-[130px] cursor-text px-2 text-center focus:outline-none"
                      max={formatDate(toDayjs)}
                      value={from}
                      onChange={(event) =>
                        handleDateChange('from', event.target.value)
                      }
                    />
                  </label>
                </div>
                <span>-</span>
                <div className="form-control ml-2">
                  <label className="input-group-sm">
                    <input
                      type="date"
                      className="input-bordered input input-sm max-w-[130px] cursor-text px-2 text-center focus:outline-none"
                      min={formatDate(fromDayjs)}
                      value={to}
                      onChange={(event) =>
                        handleDateChange('to', event.target.value)
                      }
                    />
                  </label>
                </div>
              </div>
            }
          />
          <div className="flex w-full flex-col">
            <AutoscalingHistoryHeatmap
              autoscalingHistory={autoscalingHistory}
              from={fromDayjs}
              to={toDayjs}
            />
            <div className="flex items-center justify-end px-8 py-4">
              <div className="mx-2 flex h-8 items-center">
                <div className="mx-2 flex items-center">
                  <label className="select-group-sm">
                    <select
                      value={sizePerPage}
                      onChange={handleSizePerPage}
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
                        checked={selectAll}
                        onChange={handleSelectAll}
                      />
                    </label>
                  </th>
                  <th className="mx-4 flex h-full w-full flex-4 items-center">
                    <span className="flex items-center break-words">
                      Scaling Plan ID
                    </span>
                  </th>
                  <th className="mx-4 flex h-full w-full flex-10 items-center">
                    <span className="flex items-center break-words">
                      Metric Values
                    </span>
                  </th>
                  <th className="mx-4 flex h-full w-full flex-5 items-center">
                    <span className="flex items-center break-words">
                      Metadata Values
                    </span>
                  </th>
                  <th className="mx-4 flex h-full w-full flex-3 items-center">
                    <span className="flex items-center break-words">
                      Fail Message
                    </span>
                  </th>
                  <th className="mx-4 flex h-full w-full flex-1 items-center">
                    <div className="flex items-center break-words">Status</div>
                  </th>
                  <th className="mx-4 flex h-full w-full flex-2 items-center">
                    <div className="flex items-center break-words">Date</div>
                  </th>
                  <th className="mx-4 flex h-full w-full flex-1 items-center">
                    <div className="flex items-center break-words">Actions</div>
                  </th>
                </tr>
              </thead>
              <tbody className="text-md min-h-12 flex w-full flex-col items-center justify-between py-0 text-gray-800">
                {visibleAutoscalingHistory.map(
                  (autoscalingHistoryItem: AutoscalingHistoryDefinitionEx) => (
                    <tr
                      /* id values are distinct from each other, but due to a bug, the following error occurs - Warning: Each child in a list should have a unique "key" prop. */
                      key={autoscalingHistoryItem.id}
                      className="flex w-full border-b px-8 py-4"
                    >
                      <td className="mr-4 flex h-full flex-1 items-start">
                        <label className="flex items-center">
                          <input
                            type="checkbox"
                            className="checkbox"
                            checked={autoscalingHistoryItem.isChecked}
                            onChange={(event) => {
                              const checked = event.target.checked;
                              const updatedAutoscalingHistory =
                                autoscalingHistory.map((item) =>
                                  item.id === autoscalingHistoryItem.id
                                    ? { ...item, isChecked: checked }
                                    : item
                                );
                              setAutoscalingHistory(updatedAutoscalingHistory);
                            }}
                          />
                        </label>
                      </td>
                      <td className="mx-4 flex h-full w-full flex-4 items-start">
                        <div className="flex items-center break-all">
                          {autoscalingHistoryItem.plan_id}
                        </div>
                      </td>
                      <td className="mx-4 flex h-full w-full flex-10 items-start">
                        <div className="flex flex-col items-center">
                          {renderKeyValuePairsWithJson(
                            autoscalingHistoryItem.metric_values_json,
                            true
                          )}
                        </div>
                      </td>
                      <td className="mx-4 flex h-full w-full flex-5 items-start">
                        <div className="flex flex-col items-center">
                          {renderKeyValuePairsWithJson(
                            autoscalingHistoryItem.metadata_values_json,
                            true
                          )}
                        </div>
                      </td>
                      <td className="mx-4 flex h-full w-full flex-3 items-start">
                        <div className="flex flex-col items-center">
                          {autoscalingHistoryItem.fail_message &&
                            renderKeyValuePairsWithJson(
                              autoscalingHistoryItem.fail_message,
                              true
                            )}
                        </div>
                      </td>
                      <td className="mx-4 flex h-full w-full flex-1 items-start">
                        <div className="flex items-center">
                          {autoscalingHistoryItem.fail_message ? (
                            <button className="badge-error badge bg-[#E0242E] px-2 py-3 text-white">
                              Failed
                            </button>
                          ) : (
                            <button className="badge-success badge bg-[#074EAB] px-2 py-3 text-white">
                              Success
                            </button>
                          )}
                        </div>
                      </td>
                      <td className="mx-4 flex h-full w-full flex-2 items-start">
                        <div className="flex items-center break-all">
                          {dayjs
                            .unix(
                              Number(autoscalingHistoryItem.created_at) / 1000
                            )
                            .format('YYYY/MM/DD HH:mm:ss')}
                        </div>
                      </td>
                      <td className="mx-4 flex h-full w-full flex-1 items-start">
                        <div className="flex items-center">
                          <button
                            className="badge-info badge bg-[#99E3D0] px-2 py-3 text-white"
                            onClick={() =>
                              onClickDetails(autoscalingHistoryItem)
                            }
                          >
                            Details
                          </button>
                        </div>
                      </td>
                    </tr>
                  )
                )}
              </tbody>
            </table>
          </div>
        </div>
        {detailsModalFlag ? (
          <AutoscalingHistoryDetailDrawer
            autoscalingHistoryItem={autoscalingHistoryItem}
            setDetailsModalFlag={setDetailsModalFlag}
            setFetchFlag={setFetchFlag}
          />
        ) : null}
      </div>
    </main>
  );
}
