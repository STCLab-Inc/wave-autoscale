export default class Pagination {
  pageSize: number;
  total: number;
  private _currentPage: number;

  constructor(
    pageSize: number = 0,
    total: number = 0,
    currentPage: number = 1
  ) {
    this.pageSize = pageSize;
    this.total = total;
    this._currentPage = currentPage;
    this.currentPage = currentPage;
  }

  get totalPage() {
    return Math.ceil(this.total / this.pageSize);
  }

  // Current page number
  get currentPage() {
    return this._currentPage;
  }
  set currentPage(page: number) {
    if (page < 1) {
      page = 1;
    }
    if (page > this.totalPage) {
      page = this.totalPage;
    }
    this._currentPage = page;
  }
}
