export const idlFactory = ({ IDL }) => {
  return IDL.Service({
    'balance' : IDL.Func([IDL.Nat64], [IDL.Nat], ['query']),
    'public_key' : IDL.Func([], [IDL.Text], ['query']),
    'set_pk' : IDL.Func([], [], []),
    'start' : IDL.Func(
        [
          IDL.Principal,
          IDL.Vec(IDL.Tuple(IDL.Nat64, IDL.Tuple(IDL.Text, IDL.Nat64))),
        ],
        [],
        [],
      ),
  });
};
export const init = ({ IDL }) => { return []; };
