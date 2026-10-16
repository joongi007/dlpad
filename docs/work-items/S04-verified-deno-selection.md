# S04: 관리 Deno 실제 버전 확인 후 선택

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R05, R08, R11
- 담당/수정 범위: root — tools.rs/tools/deno_version.rs 구현·examples/install_managed_archive.rs·이 문서; metrics_recovery — deno_version.rs tests만
- 선행 작업: 비활성 package/fullset selection, 공식 Linux Deno output 실증, 이전 전체workspace 두OS PASS

## 사용자 결과

관리 Deno 선택 worker는 실제 --version과 원 파일을 재검증한 뒤 활성 버전을 전환한다. 버전/종료/내용변경/취소/timeout/미지원target 실패면 이전active를유지한다. GUI 연결은 후속 단계다.

## 계약과 범위

select_verified_deno(package,cancel) API는 같은root lock 안에서 package.validate/hosttarget/fullsethash/originaldirectory+main nativepins→owned ProcessSpec argv[--version]→정확3줄/version/target parser→원ID/fullsethash 재확인→cancel/deadline/root확인→active DB transaction을 수행한다. 총budget10초, log4096 및 기존metadata capture8MiB 상한을 유지하고 parser입력4096bytes를초과하면거절한다. 새Cargo/schema추가는없다. 공급 trust/license/notice 정책 채택 또는 packagedGUI 완료와 구분한다.

개발 예제 local-verified 모드는 임시 cache 공식source만 사용해 제품 API actualactivation/active삭제거절/source보존을 검사한다. version process 종료 후 임시 fixture 전체 정리는 tempfile의 소유cache 정리이며 제품 active 제거로표현하지않는다.

## 인수 조건

순수 정확output/CRLF·잘못된prefix/version/target/banner/비UTF8/overflow와 실제process 성공/nonzero/selfmutation/취소 실패를 검사한다. 공식Linuxproduct API 선택과 삭제거절·source전후SHA를JSON에기록한다. 다른OS actualruntime/typeddownloadargv/UI/attempt진단은남는다.

## 검증과 handoff

기존전제snapshot Linux9operations/currentSHA368 PASS, Windows5operations/currentSHA370 PASS를 root가 실제확인후 sourcefreeze해제했다. 새 선택API source는별도테스트중이다.

## 실제 초기 검증

Linux tools63PASS/1ignored 및 Windows CPU전체Clippy0·tools42PASS를 root가 확인했다. 공식local-verified 제품API실증은 exit1로실패하여 합성회귀로대체하지않는다. 초기예제의고정실패문구가실제지점을생략했고 실행요청여부도잘못표시했으므로 bounded160자 개발오류표시로보강한다. [첫실패 원시](../validation/evidence/verified-deno-official-product-first/result.json)를보존한다. 실제sourcebeforeafter불변이며 제품GUI/다른OSruntime는남는다.

## 공식 실증 실패 원인

[상세 실패](../validation/evidence/verified-deno-official-product-error-detail/result.json)는 Linux guardian missing NotFound다. 예제가 release/examples 경로에서 실행되어 sibling guardian을 찾지 못했으며 제품의 self guardian 등록도 누락했다. root는 기존 guardian_entry/register_self_guardian을 예제main에연결한다. guardian 없이plain spawn으로우회하지않는다. 버전parser회귀와실제guardian환경누락을구분하고원실패를보존한다.

## 공식 제품 API 재검증 성공

[원 실제 결과](../validation/evidence/verified-deno-official-product-self-guardian/result.json)는 self guardian 등록 후 exit0/PASS다. source 전후SHA c6527f24… 동일, 실제제품select_verified_deno→active전환→remove_package ActiveVersion거절을확인했다. 전체3.69초/peak4992KiB, install2462ms 및 verification+삭제거절823ms이며 임시fixture만정리했다. 새source의전체workspace는후속검증하며 다른OS공식runtime/GUI·typedargv/Attempt·publishernotice/trust는미완료다.
