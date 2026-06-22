import assert from "node:assert/strict";
import { createServer } from "node:http";
import test from "node:test";

import {
  buildSmokeRequestPlan,
  createDryRunReport,
  determineCoverageMode,
  extractRepoIds,
  normalizeCodeSearchQuery,
  parseArgs,
  requestJsonTransport,
  renderBenchmarkReportToml,
  resolveBenchmarkReportPath,
  summariseSuite,
  summariseCoverageModes,
} from "./wendao_gateway_openapi_benchmark.ts";

test("extractRepoIds keeps only projects with non-empty plugins", () => {
  const workspaceConfig = `
[sources.projects."alpha/Alpha.jl"]
plugins = ["repo_intelligence"]

[sources.projects.beta]
plugins = []

[sources.projects.gamma]
root = "/tmp/gamma"
plugins = ["repo_intelligence", "semantic"]
`;

  assert.deepEqual(extractRepoIds(workspaceConfig), ["alpha/Alpha.jl", "gamma"]);
});

test("normalizeCodeSearchQuery strips suffixes and separators", () => {
  assert.equal(normalizeCodeSearchQuery("JuliaLang/My_Package.jl"), "JuliaLang My Package");
});

test("parseArgs returns defaults and validates overrides", () => {
  const defaults = parseArgs(["--dry-run"]);
  assert.equal(defaults.dryRun, true);
  assert.equal(defaults.concurrency, 6);
  assert.equal(defaults.stressConcurrency, 48);
  assert.equal(defaults.stressDurationMs, 30_000);
  assert.equal(defaults.help, false);

  const explicit = parseArgs(["--report-path", ".benchmark/custom.toml", "--dry-run"]);
  assert.equal(explicit.reportPath, ".benchmark/custom.toml");

  assert.throws(() => parseArgs(["--repo-limit", "0"]), /--repo-limit must be a positive integer/);
});

test("summariseSuite counts non-empty hits only when the metric exposes hit counts", () => {
  const summary = summariseSuite("code_search", [
    {
      suite: "code_search",
      label: "a",
      url: "http://example/a",
      elapsedMs: 12,
      ok: true,
      status: 200,
      hitCount: 3,
    },
    {
      suite: "code_search",
      label: "b",
      url: "http://example/b",
      elapsedMs: 24,
      ok: false,
      status: 500,
      hitCount: 0,
    },
  ]);

  assert.equal(summary.requests, 2);
  assert.equal(summary.ok, 1);
  assert.equal(summary.failed, 1);
  assert.equal(summary.nonEmptyHitCount, 1);
});

test("coverage helpers classify benchmark smoke and skipped operations", () => {
  const operations = [
    {
      method: "get" as const,
      path: "/api/search/intent",
      operationId: "searchIntent",
      parameterNames: [],
    },
    {
      method: "get" as const,
      path: "/api/health",
      operationId: "health",
      parameterNames: [],
    },
    {
      method: "post" as const,
      path: "/api/ui/config",
      operationId: "setUiConfig",
      parameterNames: [],
    },
  ];

  assert.equal(determineCoverageMode(operations[0]), "benchmark");
  assert.equal(determineCoverageMode(operations[1]), "smoke");
  assert.equal(determineCoverageMode(operations[2]), "skip");
  assert.deepEqual(summariseCoverageModes(operations), {
    benchmark: 1,
    smoke: 1,
    skip: 1,
  });
});

test("createDryRunReport exposes operation counts and benchmark paths", () => {
  const report = createDryRunReport({
    projectRoot: "/tmp/workspace",
    gatewayUrl: "http://127.0.0.1:9517",
    openapiPath: "/tmp/openapi.json",
    workspaceConfig: "/tmp/wendao.toml",
    sampledRepoIds: ["ADTypes.jl"],
    plannedSearchCases: [{ repoId: "ADTypes.jl", query: "ADTypes" }],
    openApiOperations: [
      {
        method: "get",
        path: "/api/search/intent",
        operationId: "searchIntent",
        parameterNames: [],
      },
      {
        method: "post",
        path: "/api/ui/config",
        operationId: "setUiConfig",
        parameterNames: [],
      },
    ],
  });

  assert.equal(report.openapiOperationCount, 2);
  assert.equal(report.reportDirectory, "/tmp/.benchmark");
  assert.equal(report.coverageModes.benchmark, 1);
  assert.equal(report.coverageModes.skip, 1);
  assert.deepEqual(report.stressSettings, {
    concurrency: 48,
    durationMs: 30_000,
    maxRequests: 5_000,
  });
  assert.deepEqual(report.stressSuites, ["stress_code_search", "stress_mixed_user_hotset"]);
  assert.deepEqual(report.benchmarkOperationPaths, [
    "/api/repo/index/status",
    "/api/repo/sync",
    "/api/search/index/status",
    "/api/search/intent",
  ]);
});

test("resolveBenchmarkReportPath defaults under .benchmark with timestamped TOML", () => {
  const reportPath = resolveBenchmarkReportPath(
    parseArgs(["--dry-run"]),
    { PRJ_ROOT: "/tmp/workspace" } as NodeJS.ProcessEnv,
    "/tmp/ignored",
    new Date("2026-03-24T22:40:11.123Z"),
  );

  assert.equal(
    reportPath,
    "/tmp/workspace/.data/wendao-frontend/.benchmark/wendao_gateway_openapi_2026_03_24T22_40_11_123Z.toml",
  );
});

test("requestJsonTransport reads JSON responses over node http", async () => {
  const testPort = 19517;
  const server = createServer((request, response) => {
    if (request.url !== "/api/health") {
      response.writeHead(404, { "content-type": "application/json" });
      response.end(JSON.stringify({ error: "not found" }));
      return;
    }
    response.writeHead(200, { "content-type": "application/json" });
    response.end(JSON.stringify({ ok: true }));
  });

  await new Promise<void>((resolve, reject) => {
    server.listen(testPort, "127.0.0.1", () => resolve());
    server.on("error", reject);
  });

  try {
    const response = await requestJsonTransport(
      new URL(`http://127.0.0.1:${testPort}/api/health`),
      1_000,
    );

    assert.equal(response.status, 200);
    assert.equal(response.ok, true);
    assert.equal(response.body, JSON.stringify({ ok: true }));
  } finally {
    await new Promise<void>((resolve, reject) => {
      server.close((error) => {
        if (error) {
          reject(error);
          return;
        }
        resolve();
      });
    });
  }
});

test("buildSmokeRequestPlan supplies markdown path for analysis smoke", () => {
  const plan = buildSmokeRequestPlan(
    {
      method: "get",
      path: "/api/analysis/markdown",
      operationId: "studioMarkdownAnalysis",
      parameterNames: ["path"],
    },
    {
      markdownFilePath: "README.md",
    },
    new URL("http://127.0.0.1:9517"),
    20,
  );

  assert.ok("url" in plan);
  assert.equal(plan.url.toString(), "http://127.0.0.1:9517/api/analysis/markdown?path=README.md");
});

test("buildSmokeRequestPlan uses definitionQuery for definition smoke", () => {
  const plan = buildSmokeRequestPlan(
    {
      method: "get",
      path: "/api/search/definition",
      operationId: "studioSearchDefinition",
      parameterNames: ["q"],
    },
    {
      repoQuery: "ADTypes",
      definitionQuery: "Search Parameters",
    },
    new URL("http://127.0.0.1:9517"),
    20,
  );

  assert.ok("url" in plan);
  assert.equal(
    plan.url.toString(),
    "http://127.0.0.1:9517/api/search/definition?q=Search+Parameters",
  );
});

test("renderBenchmarkReportToml records coverage, discovery, and failures", () => {
  const toml = renderBenchmarkReportToml(
    {
      gatewayUrl: "http://127.0.0.1:9517",
      openapiPath: "/tmp/openapi.json",
      workspaceConfig: "/tmp/wendao.toml",
      repoCount: 177,
      openapiOperationCount: 3,
      summaries: [
        {
          suite: "code_search",
          requests: 2,
          ok: 2,
          failed: 0,
          p50Ms: 10,
          p95Ms: 12,
          maxMs: 12,
          avgMs: 11,
          nonEmptyHitCount: 1,
        },
      ],
      stressSummaries: [
        {
          suite: "stress_code_search",
          configuredDurationMs: 30_000,
          actualDurationMs: 30_000,
          concurrency: 48,
          maxRequests: 5_000,
          requests: 200,
          ok: 198,
          failed: 2,
          successRate: 0.99,
          throughputRps: 6.67,
          avgMs: 500,
          p50Ms: 480,
          p95Ms: 900,
          p99Ms: 1100,
          maxMs: 1400,
          nonEmptyHitCount: 150,
          capped: false,
        },
      ],
      aggregateRepoIndexStatus: {
        suite: "repo_index_status",
        label: "aggregate",
        url: "http://127.0.0.1:9517/api/repo/index/status",
        elapsedMs: 2,
        ok: true,
        status: 200,
        total: 177,
        ready: 109,
      },
      aggregateSearchIndexStatus: {
        suite: "search_index_status",
        label: "aggregate",
        url: "http://127.0.0.1:9517/api/search/index/status",
        elapsedMs: 4,
        ok: true,
        status: 200,
      },
      repoIndexSnapshot: {
        total: 177,
        ready: 109,
        failed: 55,
        unsupported: 13,
      },
      searchIndexSnapshot: {
        total: 6,
        degraded: 2,
        statusReasonCode: "repo_index_failed",
      },
      discovery: {
        readyRepoId: "ADTypes.jl",
        definitionQuery: "Search Parameters",
        markdownFilePath: "README.md",
        pageId: "repo:ADTypes.jl:page:1",
      },
      coverageSummary: {
        totalOperations: 3,
        benchmarkOperations: 1,
        smokeOperations: 1,
        skippedOperations: 1,
        passedOperations: 2,
        failedOperations: 0,
      },
      operationCoverage: [
        {
          method: "get",
          path: "/api/search/intent",
          operationId: "searchIntent",
          mode: "benchmark",
          status: "passed",
          suite: "code_search",
          label: "ADTypes.jl",
          url: "http://127.0.0.1:9517/api/search/intent?q=ADTypes",
          elapsedMs: 10,
          httpStatus: 200,
        },
        {
          method: "post",
          path: "/api/ui/config",
          operationId: "setUiConfig",
          mode: "skip",
          status: "skipped",
          skipReason: "mutating OpenAPI operation is intentionally skipped in live benchmark",
        },
      ],
      failures: [
        {
          suite: "openapi_smoke",
          label: "docsSearch",
          url: "http://127.0.0.1:9517/api/docs/search?repo=ADTypes.jl",
          elapsedMs: 30,
          ok: false,
          status: 500,
          error: "boom",
        },
      ],
    },
    parseArgs(["--dry-run"]),
    "2026-03-24T22:40:11.123Z",
  );

  assert.match(toml, /schema_version = "2"/);
  assert.match(toml, /\[coverage_summary\]/);
  assert.match(toml, /ready_repo_id = "ADTypes\.jl"/);
  assert.match(toml, /definition_query = "Search Parameters"/);
  assert.match(toml, /markdown_file_path = "README\.md"/);
  assert.match(toml, /\[\[stress_summaries\]\]/);
  assert.match(toml, /stress_concurrency = 48/);
  assert.match(toml, /\[\[operation_coverage\]\]/);
  assert.match(toml, /\[\[failures\]\]/);
});
