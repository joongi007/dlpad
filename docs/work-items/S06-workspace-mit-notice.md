# S06: 선언된 workspace MIT 원문 포함

- 상태: done
- 분류: feature
- 우선순위: P0
- 요구사항: R10, R11
- 담당/수정 범위: root; LICENSE, README.md, scripts/windows_candidate_package.py, 본 작업과 신규검증
- 선행 작업: Cargo.workspace.package license=MIT 및 3개 crate license.workspace=true, 최신4명령 proof

## 사용자 결과

기존 Cargo에 선언된 프로젝트 MIT 라이선스의 원문을 저장소와 개발 후보의 각 workspace package에 포함한다.

## 계약과 범위

기존 MIT 선언을 문서화하는 가역 변경이며 새로운 라이선스로 바꾸지 않는다. Copyright는 dlpad contributors로 표기하며 실명·외부 게시·법적완비/제3자저작물권리승인을 주장하지 않는다. 제품 source/Cargo/lock/deps/실행비용0, 작은 LICENSE 원문과 inventory 기록만 추가한다. 3개 name/version/path/inheritedlicense를 정확검증하고 root LICENSE 원문을 bounded 포함한다. 기존 foreignworkspace/git 누락을 범용성공으로 변경하지 않는다. 정상 Git 없는 상태는 유지한다.

## 인수 조건

정확 dlpad/core/engine 0.1.0, 원sourcepath·Cargo MIT 선언·license inheritance 유지일때만각notice 포함. 잘못된 이름/버전/license/path 거절, notice SHA/bytes 검증. 개발 후보 재현·누락 감소 실측과독립review를통과한다. 다른 dependency notice/embedded legal gate는별도다.

## 검증과 handoff

기존 선언의 문서 보완으로 결정했고 사용자 승인 없는 공개 릴리스나 Git 변경은 수행하지 않는다.

root positive3/negative15 PASS 후 독립 검토에서 prefix/ASIS부분검증의중간문단변조공백을발견했다. 실제LICENSE원문은정상이었으며root가exact1075B/SHA와copy시재검증을추가했다. 기존prefix를보존한permission문단변조negative 실제거절+정상3PASS를확인했다. 기존2ZIP의원문은동일canonicalSHA지만실행당validator는이전버전으로기록하고보강후새후보실행을별도수행한다.

정확3workspace 원문1075B/SHA의archive entry를두ZIP 모두직접재검증했고같은SHA b5a53e58…/12,651,524B/861entries/missing44를확인했다. 생성당이전prefixvalidator였음을기록하고현재strictexactSHA·copy재검증/중간문단negative를별도확인했다. 후보ZIP독립구조review를추가확인하며전체embeddedlicense/Winapiimportlib/공개릴리스는미완료다.
