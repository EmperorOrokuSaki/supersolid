export const idlFactory = ({ IDL }) => {
  return IDL.Service({
    'balance' : IDL.Func([IDL.Nat64], [IDL.Nat], ['query']),
  });
};
export const init = ({ IDL }) => {
  return [
    IDL.Principal,
    IDL.Vec(IDL.Tuple(IDL.Nat64, IDL.Tuple(IDL.Text, IDL.Nat64))),
    IDL.Text,
  ];
};
