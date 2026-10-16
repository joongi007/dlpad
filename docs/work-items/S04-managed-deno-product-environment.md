# S04: Windows 관리 Deno 환경의 제품 다운로드 연결

- 상태: done
- 분류: feature
- 우선순위: P0
- 요구사항: R04, R05, R08, R11, R12, R13
- 담당/수정 범위: root — shared factory/policy/wire/DownloadSession/API, download/deno_runtime/app worker 통합 및 실제 Windows 전체 검증. soak_recovery_audit — 3 capture /tmp 사본. metrics_recovery — public native synthetic fixture /tmp 사본. diagnostic_cache — 독립 읽기 검토.
- 선행 작업: 원 cache identity attachment/락 인계/actual cookie helper Some/native junction 인수

## 사용자 결과

실제로 관리된 Deno 선택에만 전용 cache와 고정 node-shim opt-out 환경을 적용한다. 버전 조회와 metadata·다운로드가 동일한 원 권한을 사용하며 외부 Deno는 cache를 생성하거나 변경하지 않는다. GUI 긴 IO는 기존 worker에서 수행한다.

## 계약과 범위

factory는 전체 archive package 및 원 native proof를 retained Arc authority의 cache Mutex와 함께 보유한다. DownloadSession은 Windows 전용 optional 환경 borrow를 실행 수명 동안만 받으며 DB/options 저장에 넣지 않는다. 앱은 관리 root의 worker에서 정확한 Deno를 분류·cache 준비 후 관리 실행을 시작한다. 엔진은 환경을 사용할 Deno의 원 canonical 경로와 managed scope를 확인한다. Some capture는 원 authority의 bounded Mutex획득·binding 비교를 journal 변경 전에 수행하며 helper에는 immutable binding +원 cache.lock만 전달한다. journalrelease 뒤 explicitcacheunlock/AppAck를 보내고 실제 helper Terminal/EOF/exit 뒤 cache 재획득이 성공해야 journal resolve/reader confirm을 진행한다. cache 실패는 pending/원본 보존이며 원경로에서 권한을 재생성하지 않는다.

## 인수 조건

관리 Deno에만 version/metadata/download Some 환경 적용, external None/no cache creation, 원 root ID 불일치/partial cache/취소 거절과 원 bytes 보존. actual managed-only/paired synthetic 다운로드의 성공·단계실패·취소, 같은 worker의 순차 cache reacquire와 마지막 scope finish/reuse 및 raw cookie 비저장을 실제 Windows에서 확인한다. actual EJS/site/official 공급 채택·GUI·appkill 인수는 별도이며 합성 실행으로 완료를 주장하지 않는다. Windows 실제 fmt/Clippy/workspace tests/release 및 문서 검사와 원 source SHA 대조를 수행한다.

## 검증과 handoff

구현 통합과 actual native 인수는 진행 중이다. staged 구현이나 앞선 cookie helper 직접시험만으로 public 제품 경로 완료를 표시하지 않는다. 의존성/Cargo.lock/DB schema 변경은 없다.

## 주 담당 통합 착수 기록

factory 사본2개와 capture 사본3개를 원 프로젝트 SHA/changed SHA manifest 대조 뒤 게시했다. 공유 native wire는 immutable original binding만 clone하며 Mutex를 중복 획득하지 않는다. helper 복원은 expected binding을 유지하되 parent authority를 만들어내지 않는다. 앱 기존 worker와 Windows-only DownloadSession borrow를 연결했고, Deno inspector는 canonical selected executable과 원 factory proof 경로를 대조한다. metadata/download와 Deno version probe는 같은 정책을 clone하여 순차 cache 인계를 사용한다. 원 caller15개에 None 기본값을 추가해 다른 플랫폼·외부 경로 계약을 유지했다.

독립 검토의 EOF 이후 '무제한 wait' 주장은 root가 실제 `windows.rs::Child::wait`의 fixed TERMINAL_GRACE 및 원 HANDLE Drop/유한 정리를 확인해 정정했다. 별도 bounded cleanup grace가 실행 deadline에 추가될 수는 있지만 이 경로가 무한 wait라는 근거는 없다. 계획/추정만으로 불필요한 공유 프로세스 개편을 하지 않았다. actual public5scenario fixture는 원 staging SHA `4524774d63c2c1ee0c260fce38ba0a66f2624b46c94d89a760c6943fd7869ba9`를 대조하여 새 example에 게시했으며 컴파일/native 실행 인수는 이후 수행한다.

## 최초 공개 경로 Windows 실행 결과

[실제 실행 증거](../validation/evidence/windows-managed-deno-product-20261004T121154Z/result.json)에서 fresh helper/example 빌드, factory native3개, 성공·paired 합성 쿠키·버전 불일치·metadata 실패·취소5case가 모두 exit0이다. [주 담당 대조](../validation/evidence/windows-managed-deno-product-20261004T121154Z/root-review.json)는 실제 원 receipt bytes/SHA/magic, cache owner SHA와 pending 부재를 각 소유 fixture에서 다시 확인했다. 실제 Deno/FFmpeg·사이트·GUI 시험은 아니다.

첫 전체 workspace 시험은 fmt 통과 후 새 example의 Clippy cmp_owned/collapsible_if 두 건으로 실패했다. 실패 로그는 windows-managed-deno-product-workspace-20261004T121430Z에 보존했다. 두 문구를 의미 유지 형태로 수정했으며 현재 source의 전체 검증은 새 collector에서 진행 중이다. 앞선5case 결과는 수정 전 snapshot으로 구분하며 최신 전체 완료를 주장하지 않는다.

## 수정 후 Windows 전체 검증

[최신 전체 검증](../validation/evidence/windows-managed-deno-product-workspace-corrected-20261004T121546Z/result.json)은 fmt/Clippy/workspace test/release 모두 exit0이다. root-review.json에서 현재 전체 source SHA와 release CPU artifact SHA를 다시 대조했다. 최신 helper/example 재빌드 및5case 재실행은 진행 중이며 완료 후 별도 결과를 기록한다. 전체 P0/GUI/실제 공급 및 사이트 인수 완료를 의미하지 않는다.

## 최신 source 재실행 확정

[최신5case](../validation/evidence/windows-managed-deno-product-corrected-20261004T121739Z/result.json)는 fresh helper/example build와 성공·paired-cookie·version-mismatch·metadata-failure·cancel 모두 exit0이다. root-review.json은 각 operation source SHA가 현재 source와 같고 원 receipt magic/256bytes/SHA, owner SHA, pending 부재를 다시 확인했다. 최신 Windows workspace 네 명령 PASS 근거와 결합해 이 합성 제품 연결 인수는 완료다. 실제 Deno/EJS/site·GUI·부모kill·공급/패키징은 후속 P0로 남는다.
