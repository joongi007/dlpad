# S04: 비파괴 도구 실행 가능 검사

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R02, R08
- 담당/수정 범위: root — shared API/direct_download.rs 통합·검증; installer_batch — selection.rs/deno_version.rs/ffmpeg_version.rs 후보
- 선행 작업: Windows 관리 설치·선택·목록 identity 및 기존 strict tool verifier

## 사용자 결과

기존 원 root lock·whole package/hash·버전 조회·소유 process·10초 취소 기한을 재사용하되 검사에서는 activation DB를 쓰지 않는다. active와 pinned inactive의 검사 결과를 UI로 제공한다.

## 계약과 범위

공유 schema/Cargo·계약은 root가 조정한다. 동시에 같은 파일을 수정하지 않고 후보는 프로젝트 소유 영구 recovery 디렉터리에 보존한다. 기존 /tmp 준비물은 WSL 재시작 때 소실됐으며 옛 proof를 복원했다고 표현하지 않는다.

## 인수 조건

정상 검사·변조·잘못된 타겟·version 오류·취소·pending에서 원 active/사용자 파일을 보존한다. 실제 Windows 실패 경로와 기존 selector 회귀를 함께 실행한다.

## 검증과 handoff

현재 후보 구현·API 합의 중이다. 아직 제품 통합/실제 PASS가 아니다. v19 IME 전체 검증 재개와 병행하되 source freeze 동안 새 tool 후보를 live에 채택하지 않는다.

## Windows 합성 도구 setup 실패 보존

`managed-tool-isolated-core-green-20261013`은 Rust 컴파일을 통과했으나 첫 시험 fixture용 C# compiler가 exit1을 반환하여 제품 검사 전에 실패했다. 0 PASS/1 FAIL을 보존하며 제품 정책 RED나 실제 verifier PASS로 확대하지 않는다. 담당자가 별도 fixture 수정 후보를 준비하고 root가 다시 실행한다.

## 재시험 결과

`managed-tool-isolated-core-diagnostic-v3-20261013`에서 legacy CSC가 `\\?\` 경로 표기에 CS2021을 반환함을 실제 관측했다. 소유된 짧은 fixture의 컴파일 argv만 정상 drive 표기로 변환한 v4를 별도 준비했으며 생산 경로 identity/제약은 유지했다. `managed-tool-isolated-core-green-v4-20261013`의 실제 Windows 합성 verifier 및 SQLite busy ACK 후 취소 롤백 ignored 시험 2개는 2 PASS/0 FAIL이고 root가 원시 host/source/candidate/actual1 결과를 독립 확인했다. v2/v3 setup 실패를 보존한다. 아직 제품 live 통합/전체 workspace 검증 완료가 아니다.

## 제품 소스 채택 checkpoint

root가 8개 격리 검증 파일을 제품 소스에 채택했다. 원 v19 8파일 backup·각 original/accepted hash·중간 intent/receipt는 `.local-tools/session-recovery-20261013/live-adoption-v20/`에 보존했다. Catalog fmt만 후속 적용했다. `windows-msvc-managed-tool-auto-whole-20261013`에서 새 제품 fmt/clippy/test/release를 실제 실행 중이다. 완료 전에는 새 전체 PASS·설치본 갱신·전체 UI 또는 최신 업데이트 완료로 보고하지 않는다.

## 2026-10-13 v20 통합 검증 완료 범위

Windows MSVC `windows-msvc-managed-tool-auto-whole-20261013`의 fmt/clippy/workspace test/release build는 모두 통과했다(1357 PASS/0 FAIL/37 ignored). root가 whole 원본 로그·657 source map·현재 실행파일 3개의 정확한 SHA/bytes를 독립 확인했다. v20 앱 63 PNG는 root11+하위52 분담 직접 검수 및 현재 image pin 일치를 확인했고, installer 주요8 PNG와 정상/음수 launch raw lifetime/source/input pin도 확인했다. 새 다운로드 환경 설명 배치는 별도 probe geometry 오차가 있어 미인수다. P1 최신 조회/동적 공급/UI 통합은 후보이며 이 설치본에 포함되지 않는다. 전체 P0·물리 IME·정식 서명/공개 배포 완료를 뜻하지 않는다.
