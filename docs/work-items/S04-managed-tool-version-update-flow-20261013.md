# S04: 도구 업데이트·버전 지정 흐름

- 상태: in-progress
- 분류: feature
- 우선순위: P1
- 요구사항: R02, R08, R11
- 담당/수정 범위: root — 정책/인터페이스/통합; 하위 담당은 선행 자동 선택 완료 후 배정
- 선행 작업: Windows 관리 설치·선택·목록 identity 및 기존 strict tool verifier

## 사용자 결과

앱 updater와 도구 updater를 구분한다. 설치된 버전 선택·전환 기반을 유지하고 최신 확인·검증된 버전 설치·되돌리기 UX를 이어서 구현한다. 임의 unsigned manifest를 실행 가능 공급으로 자동 신뢰하지 않는다.

## 계약과 범위

공유 schema/Cargo·계약은 root가 조정한다. 동시에 같은 파일을 수정하지 않고 후보는 프로젝트 소유 영구 recovery 디렉터리에 보존한다. 기존 /tmp 준비물은 WSL 재시작 때 소실됐으며 옛 proof를 복원했다고 표현하지 않는다.

## 인수 조건

업데이트 실패 시 기존 버전 유지, 진행 중 다운로드 pin 유지, 버전 선택/원본 보존/재실행 검증. 공개 배포·외부 게시·실제 계정 인증은 별도다.

## 검증과 handoff

현재 후보 구현·API 합의 중이다. 아직 제품 통합/실제 PASS가 아니다. v19 IME 전체 검증 재개와 병행하되 source freeze 동안 새 tool 후보를 live에 채택하지 않는다.

## 병렬 담당과 실제 발견

`installer_batch`는 새 recovery 후보의 최신 release 관측·승인된 특정 버전 설치·검증된 기존 버전 복귀 backend를 맡는다. 후보 경로는 `.local-tools/session-recovery-20261013/tool-update-flow-v1/`이며 제품 live 파일 수정은 root 통합 때 수행한다. 자동선택 worker `windows_auto`와 파일을 분리한다. 기존 공급은 yt-dlp 2026.08.19, Deno 2.9.7, FFmpeg 2026-09-30 고정 후보이므로 현재 최신 설치가 완성됐다고 보고하지 않는다. 동일 release checksum 관측은 독립 publisher trust로 승격하지 않는다. yt-dlp 기존 download 성공은 active를 바꾸므로 install-only API 분리 선행을 root가 담당한다.

## Install-only 기반 실제 검증

root가 `direct_download.rs` 후보에 install-only 공개 API를 분리했다. 기존 활성 설치 API는 유지하고 원본 verified_ytdlp_version 검사와 precommit 취소를 공유한다. `managed-tool-isolated-inactive-green-v3-20261013`의 actual Windows ignored 2개는 2 PASS/0 FAIL: active INSERT/UPDATE/DELETE 모두 거부하는 DB에서도 inactive 설치가 성공하고, hash/version/transport/read-cancel/SQL 실패 때 기존 active와 설치 bytes를 보존한다. 별도 actual active UPDATE hook 직후 취소는 transaction rollback을 확인했다. 이전 test-only manifest Eq 컴파일 실패는 보존하고 shared schema에 Eq를 추가하지 않았다. 후보 전체 Windows all-targets strict clippy는 v2에서 실제 PASS이며 최초 두 lint 실패도 보존한다. 아직 UI 업데이트 Action·최신 버전 공급·제품 live 반영 완료가 아니다.

## 제품 소스 채택 checkpoint

root가 8개 격리 검증 파일을 제품 소스에 채택했다. 원 v19 8파일 backup·각 original/accepted hash·중간 intent/receipt는 `.local-tools/session-recovery-20261013/live-adoption-v20/`에 보존했다. Catalog fmt만 후속 적용했다. `windows-msvc-managed-tool-auto-whole-20261013`에서 새 제품 fmt/clippy/test/release를 실제 실행 중이다. 완료 전에는 새 전체 PASS·설치본 갱신·전체 UI 또는 최신 업데이트 완료로 보고하지 않는다.

## 2026-10-13 v20 통합 검증 완료 범위

Windows MSVC `windows-msvc-managed-tool-auto-whole-20261013`의 fmt/clippy/workspace test/release build는 모두 통과했다(1357 PASS/0 FAIL/37 ignored). root가 whole 원본 로그·657 source map·현재 실행파일 3개의 정확한 SHA/bytes를 독립 확인했다. v20 앱 63 PNG는 root11+하위52 분담 직접 검수 및 현재 image pin 일치를 확인했고, installer 주요8 PNG와 정상/음수 launch raw lifetime/source/input pin도 확인했다. 새 다운로드 환경 설명 배치는 별도 probe geometry 오차가 있어 미인수다. P1 최신 조회/동적 공급/UI 통합은 후보이며 이 설치본에 포함되지 않는다. 전체 P0·물리 IME·정식 서명/공개 배포 완료를 뜻하지 않는다.
