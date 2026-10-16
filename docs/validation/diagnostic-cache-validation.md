# S10 P0 진단 보관 정책 검증

2026-10-01. 범위: R13의 로컬 보고서 보관·파일 보존. [작업 계약](../work-items/S10-cache-policy.md).

## 채택한 현재 정책

자동 진단 disk cache를 사용하지 않는다. 자동 cache 비용은 **0 byte**이며 보고서는 현재 UI 세션의 메모리에만 존재한다. 앱 종료 후 실패 보고서 자체를 자동 복원하지 않는다. 작업 journal/Attempt 증거는 별도 도메인 저장소이며 이 정책으로 삭제하거나 만료시키지 않는다.

`reports::attach_local_mpd`는 사용자가 선택한 로컬 파일을 읽어 정제한 구조를 메모리 보고서에 추가한다. 원본을 다른 파일로 복사하지 않고 수정·삭제하지 않는다. `DiagnosticReport`의 미리보기·로그 편집/제외·첨부 제외는 메모리 연산이다. app은 `Option<Arc<DiagnosticReport>>`로 보고서를 보관하며 attachment worker와 export worker를 구분한다. 네트워크 수집 또는 전송은 이 경로에 없다.

지속 저장은 `reports::save_report`를 통해 사용자가 지정한 정확한 경로에 새 파일을 생성하는 경우뿐이다. 기존 파일은 덮어쓰지 않는다. 내보내기는 사용자 소유이며 앱의 cache 청소·만료 대상이 아니다. 쓰기 실패 후 Unix에서는 방금 생성한 동일 dev/ino의 regular 파일만 정리한다. 다른 OS는 identity를 확신하지 못하는 실패 산출물을 보존한다. 일반 사용자 파일·외부 설치·다운로드·쿠키를 정리하지 않는다.

진단 설계의 20 MiB/7일 값은 향후 persistent cache를 도입할 때의 제안이다. 현재 없는 cache를 만들거나 user export를 7일 뒤 삭제하는 정책으로 해석하지 않는다. cache를 새로 도입할 경우 앱 소유 root/identity, 파일당·합계·기간 상한, 시작/실패 청소, 정제본만 저장하는 조건을 별도 작업으로 구현·검증해야 한다.

## 비용과 상한

새 의존성·schema·백그라운드 timer는 없다. 보고서는 최대 2 MiB, MPD 읽기는 최대 1 MiB+초과 감지 1 byte다. MPD 첨부/생략 합계는 core에서 8개, 로그는 128행/행당 4096 byte로 제한한다. 미리보기·복사본의 일시 메모리 비용은 자동 disk cache와 별개이며, report drop은 이미 복사한 OS clipboard를 지우지 않는다. clipboard와 user export의 보관은 사용자 관리다. 임의 로그의 완전한 익명화를 보장하지 않는다.

## 실제 회귀 시험

`crates/engine/tests/diagnostic_cache.rs`는 tempfile의 합성 fixture만 사용한다.

- 첨부→정제→MD/JSON 미리보기→로그 편집/제외→첨부 제외→report drop 전후 전체 파일 경로/내용 snapshot이 동일하다. 원본 MPD, 기존 사용자 export, 합성 job journal/partial이 보존되고 cache 파일이 생성되지 않는다.
- 기존 export 충돌은 `AlreadyExists`이며 내용이 보존된다. 명시한 MD/JSON 두 파일만 생성되고 secret fixture는 생략된다. JSON 구문과 Linux 파일 권한 0600을 확인한다. 다음 세션 report를 생성하고 해제해도 이전 export가 그대로 남는다.
- 손상 XML·DTD/external entity·1 MiB 초과 첨부는 거절하고 원본 fallback이 없다. 없는 부모 디렉터리로 export하는 실패도 새로운 파일·디렉터리를 만들거나 기존 자료를 변경하지 않는다.

## 실행 결과와 남은 검증

`cargo test -p dlpad-engine --test diagnostic_cache --locked` 3/3 통과(0.01초), `cargo test -p dlpad-engine --lib reports::tests --locked` 4/4 통과(0.05초). 전용 `cargo clippy -p dlpad-engine --test diagnostic_cache --locked -- -D warnings`와 두 Rust 파일의 rustfmt check를 통과했다. `python3 scripts/check_docs.py`는 병렬 작업 중 62 문서/로컬 링크를 확인했다. 전체 workspace 통합 검증은 주 담당이 수행한다. Linux/WSL에서 filesystem 동작을 시험했으며 Windows/macOS 실행·파일 identity·clipboard 정책 검수는 완료하지 않았다. 사용자 Documents, 실제 미디어, 실제 계정·쿠키·네트워크를 사용하지 않았다. 전체 P0/출시 완료를 뜻하지 않는다.
