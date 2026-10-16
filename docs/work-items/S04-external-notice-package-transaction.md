# S04: 별도 고정 HTTPS notice의 동일 설치 트랜잭션

- 상태: done
- 분류: feature
- 우선순위: P0
- 요구사항: R05, R08, R13
- 담당/수정 범위: root archive_download.rs/bounded_zip.rs/tools.rs/shared schema·constructors·통합; metrics_recovery /tmp archive_manifest.rs 초안만; diagnostic_cache /tmp archive_install.rs 초안만; soak_recovery_audit /tmp 라이선스 자료만. 프로젝트 게시와 Cargo/lock 조정은 root만.
- 선행 작업: v1/v2 archive 설치·원 native pin·단일 install lock·HTTPS bounded transport; 현재 기본 증거 동결 완료.

## 사용자 결과

공식 runtime ZIP에 없는 notice를 고정 HTTPS 자료로 함께 검증하고 모든 구성요소가 성공한 뒤 inactive package로 게시한다. notice 실패 시 실행파일만 설치하거나 기존 활성 도구를 변경하지 않는다.

## 계약과 범위

명시 v3 ZIP/layoutNone에 외부 notice component/source_url/revision을 추가한다. v1/v2 JSON bytes와 TAR layout은 유지한다. role Notice만 최대2개/각1MiB/합2MiB, 기존 HTTPS credentials/query/fragment 거절과 정확40hex revision, 기존 component SHA/bytes에 고정한다. executable은 archive에서만 공급한다. 원file capability·하나의 root lock·같은120초 deadline·64KiB stream·소유 stage journal·전체검증·create-only publication을 재사용한다. 새 Cargo dependency0이며 런타임 비용은 notice transfer 최대2MiB와 최대2파일·원handle 및 SHA검증이다. 공급 trust/전체notice 채택은 별도이며 자동 설치 버튼을 근거 없이 활성화하지 않는다.

## 인수 조건

v1/v2 저장호환 및 unknown schema/source/unsafe URL/role/count/revision/size 거절. 정확 archive+notice 설치 후 전체set verify/reopen/remove, prioractive/source/sentinel 유지. short/extra/hash/http/cancel/deadline/foreign collision과 capability mismatch/swap은 게시 전 거절. Windows 원native 실제회귀 및 최신 전체fmt/clippy/test/release를 root 확인한다. 합성transport와 공식 HTTPS실증을 구분한다.

## 검증과 handoff

구현 중. 좁은 기존 package 검증을 실제 공식 runtime 공급 완료로 주장하지 않는다.

## 원 Windows 검증 지점

원tools106회귀 PASS 뒤 v3전송 성공1/2notice·10실패·원capability 교체/이름불일치 회귀를 통합했다. 최초 전체Clippy의 기존한줄example 생성자 필드누락을 고쳐 원실패를 보존하고 fullfmt/clippy/test/release4 PASS를 확인했다. 이후고정example플래그와새SQLitetest 변경은 후속 전체 검증 대상이다.

공식Deno2.9.7 ZIP42630221B/exe97462048B와 exactrevision LICENSE1074B의 actual public HTTPS→inactive→fresh manager whole-set→owned notice same-length tamper rejection/restore→exactremove/sentinel보존이7669ms PASS다. 첫host無argv사용오류는보존했고collector에고정--fixed-notice-experiment플래그를넣어교정했다. [원 result](../validation/evidence/windows-official-deno-notice-fixed-argv-20261005-resume/result.json), [root 원로그·파일 검토](../validation/evidence/windows-official-deno-notice-fixed-argv-20261005-resume/root-review.json), [독립 원DB·tools 검토](../validation/evidence/windows-official-deno-notice-fixed-argv-20261005-resume/independent-review.json). 삭제뒤 payload재해시는주장하지않으며원실행receipt의설치시SHA와삭제후원DB active/staging0를구분한다. 공급 attestation/transitive완비/실제runtime실행·활성화/GUI자동설치는별도다.

고정example 플래그·createjob FULL회귀 포함 현재 default CPU4전체PASS/current535source SHA불일치0를 root직접 확인했다. [원전체](../validation/evidence/windows-notice-sqlite-current-default-20261005-resume/result.json). 신규 runtime 자동설치 GUI·전체 공급trust/notice는 별도 미완료이며 이번 완료는 public 전송→inactive게시→DB재열기·검증·제거 수직API 범위다.
