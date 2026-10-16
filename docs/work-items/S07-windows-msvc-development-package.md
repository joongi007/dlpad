# S07: MSVC 개발 후보 패키지

- 상태: done
- 분류: feature
- 우선순위: P0
- 요구사항: R11, R08
- 담당/수정 범위: root — 공유 proof·package 계약·통합·검증/문서; diagnostic_cache — /tmp scripts/windows_msvc_candidate_package.py 초안; 독립 검토는 통합 후 배정
- 선행 작업: source576 MSVC 전체4검사·GUI 실행/재시작·CLI 실패 경로, 현재 MSVC app/engine notice graph 수집과 exact notice 대조.

## 사용자 결과

검증한 MSVC 앱과 개발용 로컬 설치 CLI를 같은 대상 notice와 함께 로컬 ZIP으로 보관할 수 있다.

## 계약과 범위

기존 GNU package와 schema는 유지하고 별도 MSVC packager를 만든다. 정확한 현재 CPU/CLI·whole/graph proof를 명시하며 registry lock checksum/source와 selected graph notice entries를 결합한다. proof/lock/source/원 executable 보존, binaries streaming·256MiB/notice16MiB/4096entries·경로 제한·고정 ZIP timestamp/mode를 유지한다. 동일 입력 두 ZIP으로 결정성을 확인한다. 개발 후보/release_ready=false이며 법률·static CRT/std 전체고지·서명·설치·cleanOS는 완료하지 않는다. 외부 게시나 public release는 하지 않는다.

## 인수 조건

대상 MSVC·actual4체크·현재 source map·CPU/CLI SHA/bytes·PE GUI/Console 일치, graph exact258 packages의 notice missing없음(실제 graph 결과에 따라 갱신), ZIP member 전체 SHA/bytes·두 ZIP 일치·내장폰트 declaration/notice 연결·원 입력 불변. 잘못된 target/proof/artifact와 missing reachable notice를 거절하고 원 실패를 보존한다. root가 별도 읽기검증과 독립 검토를 수행한다.

## 검증과 handoff

실제 graph는 app258/engine86/union258로 수집됐으며 notice 대조와 packager 초안을 진행한다.

실제 두 ZIP 생성 명령과 root/독립 전체member 검증을 완료했다. [root 결과](../validation/evidence/windows-msvc-notice-graph-20261006-continue/package-root-completion-review.json)와 독립 보고를 보관했다. 두 SHAefbe0fe88b036a5b90601c7dd0a685c55579dffc9b51e2f60e0384409b0a8af1/12,053,403B·464entries·nonbinary2,486,005B·selected258notice records/missing0가 일치하며 원 source576·CPU/CLI·proof·GNU/lock 보존을 대조했다. 새로운 package 실패 회귀4개와 기존 GNU companion 회귀14개도 실제 통과했다. whole lock 별도 누락43·font linked payload/static CRT/std/legal/installer/서명/cleanOS 미완료를 유지한다. 이 ZIP은 로컬 개발 후보이며 공개 배포하지 않았다.
